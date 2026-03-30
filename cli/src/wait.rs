use agent_click_core::element;
use agent_click_core::selector::SelectorChain;
use agent_click_core::{AccessibilityNode, Error, Platform};
use std::time::{Duration, Instant};

pub async fn poll_for_element(
    platform: &dyn Platform,
    chain: &SelectorChain,
    timeout: Duration,
    interval: Duration,
) -> agent_click_core::Result<AccessibilityNode> {
    let start = Instant::now();

    loop {
        match find_by_chain(platform, chain).await {
            Ok(results) if !results.is_empty() => {
                return Ok(results.into_iter().next().unwrap());
            }
            _ => {}
        }

        if start.elapsed() >= timeout {
            let selector_desc = format!("{:?}", chain);
            return Err(Error::Timeout {
                seconds: timeout.as_secs_f64(),
                message: format!("element not found matching {selector_desc}"),
            });
        }

        tokio::time::sleep(interval).await;
    }
}

pub async fn poll_for_one_element(
    platform: &dyn Platform,
    chain: &SelectorChain,
    timeout: Duration,
    interval: Duration,
) -> agent_click_core::Result<AccessibilityNode> {
    let start = Instant::now();

    loop {
        match find_one_by_chain(platform, chain).await {
            Ok(node) => return Ok(node),
            Err(Error::ElementNotFound { .. }) => {}
            Err(e) => return Err(e),
        }

        if start.elapsed() >= timeout {
            return Err(Error::Timeout {
                seconds: timeout.as_secs_f64(),
                message: format!("element not found matching {:?}", chain),
            });
        }

        tokio::time::sleep(interval).await;
    }
}

pub async fn find_by_chain(
    platform: &dyn Platform,
    chain: &SelectorChain,
) -> agent_click_core::Result<Vec<AccessibilityNode>> {
    let first = &chain.selectors[0];

    if let Some(ref path) = first.path {
        if let Some(node) = resolve_by_path(platform, first, path).await? {
            tracing::debug!("path resolution succeeded for {:?}", first.name);
            return Ok(vec![node]);
        }
        tracing::debug!("path resolution stale, falling back to search");
    }

    if chain.selectors.len() > 1 {
        return find_by_chain_in_tree(platform, chain).await;
    }

    let results = platform.find(first).await?;
    Ok(apply_index(first, results))
}

const CHAIN_TREE_DEPTH: u32 = 15;

async fn find_by_chain_in_tree(
    platform: &dyn Platform,
    chain: &SelectorChain,
) -> agent_click_core::Result<Vec<AccessibilityNode>> {
    let first = &chain.selectors[0];
    let depth = first.max_depth.unwrap_or(CHAIN_TREE_DEPTH);
    let tree = platform.tree(first.app.as_deref(), Some(depth)).await?;

    let mut results: Vec<&AccessibilityNode> = tree.find_all(&|n| first.matches(n));
    if let Some(i) = first.index {
        results = results.into_iter().nth(i).into_iter().collect();
    }

    for selector in &chain.selectors[1..] {
        let mut next_results = Vec::new();
        for node in &results {
            next_results.extend(node.find_all(&|n| selector.matches(n)));
        }
        if let Some(i) = selector.index {
            next_results = next_results.into_iter().nth(i).into_iter().collect();
        }
        results = next_results;
    }

    Ok(results.into_iter().cloned().collect())
}

fn apply_index(
    selector: &agent_click_core::Selector,
    results: Vec<AccessibilityNode>,
) -> Vec<AccessibilityNode> {
    match selector.index {
        Some(i) if i < results.len() => vec![results.into_iter().nth(i).unwrap()],
        Some(_) => vec![],
        None => results,
    }
}

async fn resolve_by_path(
    platform: &dyn Platform,
    selector: &agent_click_core::Selector,
    path: &[usize],
) -> agent_click_core::Result<Option<AccessibilityNode>> {
    let tree = platform.tree(selector.app.as_deref(), None).await?;

    let node = match tree.walk_path(path) {
        Some(n) => n,
        None => return Ok(None),
    };

    if selector.matches(node) {
        Ok(Some(node.clone()))
    } else {
        Ok(None)
    }
}

pub async fn find_one_by_chain(
    platform: &dyn Platform,
    chain: &SelectorChain,
) -> agent_click_core::Result<AccessibilityNode> {
    let mut results = find_by_chain(platform, chain).await?;
    match results.len() {
        0 => Err(Error::ElementNotFound {
            message: "no element found matching selector chain".to_string(),
        }),
        1 => Ok(results.into_iter().next().unwrap()),
        _ => {
            results.sort_by_key(|n| std::cmp::Reverse(element::rank(n)));
            let best_rank = element::rank(&results[0]);
            let tied_count = results
                .iter()
                .take_while(|n| element::rank(n) == best_rank)
                .count();

            if tied_count == 1 || best_rank > element::rank(&results[1]) {
                tracing::debug!(
                    "ambiguity resolved: picked {:?} (rank {:?}) over {} others",
                    results[0].name,
                    best_rank,
                    results.len() - 1
                );
                Ok(results.into_iter().next().unwrap())
            } else {
                tracing::debug!(
                    "ambiguity unresolvable: {} elements tied at rank {:?}",
                    tied_count,
                    best_rank
                );
                Err(Error::AmbiguousSelector { count: tied_count })
            }
        }
    }
}
