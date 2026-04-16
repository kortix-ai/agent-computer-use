---
name: agent-computer-use
description: Control any desktop app (macOS, Linux, Windows, Electron) from the command line via the agent-cu CLI. Use this skill when the user wants to automate desktop UI, script a native app, build an agent that operates a computer, or read/manipulate any visible app via accessibility APIs. Triggers on tasks involving clicking buttons, typing into fields, reading screen state, driving Music/Calculator/Safari/Finder/Slack, computing values in apps, playing media, filling forms, moving or resizing windows, or anything where the agent needs to operate a Mac/Linux/Windows app like a human would.
license: MIT
metadata:
  author: kortix-ai
  version: '0.1.0'
  homepage: https://github.com/kortix-ai/agent-computer-use
---

# agent-computer-use — Computer use Skill

You have access to `agent-cu`, a CLI tool that controls desktop applications. You can click buttons, type text, read screens, scroll, drag files, move windows — all from the terminal.

## How to think

**Think like a human sitting at the computer.** Before you act, ask yourself: what would I see on screen? What would I click? What would I type?

A human:

1. Looks at the screen (snapshot)
2. Finds what they need (identify refs)
3. Does one action (click, type, key)
4. Checks what changed (re-snapshot)

You must do the same. Never skip steps. Never assume the UI didn't change after an action.

## Core loop

```
snapshot → identify → act → verify
```

```bash
agent-cu snapshot -a Music -i -c          # what's on screen?
# read the output, find the right @ref
agent-cu click @e5                         # do one thing
agent-cu snapshot -a Music -i -c          # what changed?
```

**Every action changes the UI.** Your previous refs are now stale. Always re-snapshot.

## Opening apps

Always wait for the app to be ready before doing anything:

```bash
agent-cu open Safari --wait
agent-cu snapshot -a Safari -i -c
```

Never interact with an app you haven't opened and snapshotted first.

## Finding elements

**Step 1**: Snapshot with `-i -c` (interactive + compact):

```bash
agent-cu snapshot -a Calculator -i -c
```

This shows only clickable/typeable elements with refs like `@e1`, `@e5`, `@e12`.

**Step 2**: Read the output. Find the element you need by its name, role, or id.

**Step 3**: Use the ref. Refs are the fastest and most reliable way to target elements.

If elements are missing, increase depth:

```bash
agent-cu snapshot -a Safari -i -c -d 8
```

## Clicking

For buttons, links, menu items — use `click`:

```bash
agent-cu click @e5                         # single click (AXPress, headless)
agent-cu click @e5 --count 2               # double-click (opens files, plays songs)
```

`click` tries AXPress first (background, no focus steal). Only falls back to mouse simulation for double-click or right-click.

For elements with stable IDs (won't change between snapshots):

```bash
agent-cu click 'id="play"' -a Music
agent-cu click 'id~="track-123"' -a Music  # partial id match
```

## Typing

**With a target element** (preferred — uses AXSetValue, headless):

```bash
agent-cu type "hello world" -s @e3
```

**Into the focused field** (keyboard simulation, needs app focus):

```bash
agent-cu type "hello world" -a Safari
```

Always prefer `-s @ref` when you have a ref. It's more reliable.

## Key presses

```bash
agent-cu key Return -a Calculator
agent-cu key cmd+k -a Slack
agent-cu key cmd+a -a TextEdit
agent-cu key Escape -a Safari
```

## Scrolling

```bash
agent-cu scroll down -a Music              # scroll main content area
agent-cu scroll down --amount 10 -a Music  # scroll more
agent-cu scroll-to @e42                    # scroll element into view (headless)
```

Scroll needs the app to be focused. Use `scroll-to` for headless.

## Reading content

```bash
agent-cu text -a Calculator                # all visible text
agent-cu get-value @e5                     # one element's value/state
agent-cu get-value 'id="title"' -a Music   # by selector
```

Use `get-value` on specific elements instead of `text` on large apps.

## Window management

```bash
agent-cu move-window -a Notes --x 100 --y 100
agent-cu resize-window -a Notes --width 800 --height 600
agent-cu windows -a Finder                 # get window positions and sizes
```

These are instant and headless — use AXSetPosition/AXSetSize.

## Drag and drop

Drag needs the app to be focused and two visible, non-overlapping areas.

**Think like a human**: you need to see both the source and destination.

```bash
# Step 1: Set up windows side by side
agent-cu move-window -a Finder --x 0 --y 25
agent-cu resize-window -a Finder --width 720 --height 475
# (open a second Finder window for destination)

# Step 2: Snapshot to find the file
agent-cu snapshot -a Finder -i -c -d 8

# Step 3: Get the file's position
agent-cu get-value @e32                    # check position

# Step 4: Drag to destination
agent-cu drag @e32 @e50 -a Finder         # drag by refs
# or by coordinates:
agent-cu drag --from-x 300 --from-y 55 --to-x 1000 --to-y 200 -a Finder
```

## Selector syntax

### Refs (always prefer these)

```bash
@e1, @e2, @e3                         # from most recent snapshot
```

### DSL

```bash
'role=button name="Submit"'            # role + exact name
'name="Login"'                         # exact name
'id="AllClear"'                        # exact id (most stable)
'id~="track-123"'                      # id contains (case-insensitive)
'name~="Clear"'                        # name contains (case-insensitive)
'button "Submit"'                      # shorthand: role name
'"Login"'                              # shorthand: just name
'role=button index=2'                  # 3rd match (0-based)
'css=".my-button"'                     # CSS selector (Electron apps only)
```

### Chains

```bash
'id=sidebar >> role=button index=0'    # first button inside sidebar
'name="Form" >> button "Submit"'       # submit button inside form
```

## Electron apps (CDP)

Electron apps (Slack, Cursor, VS Code, Postman, Discord) are automatically detected. agent-cu relaunches them with CDP support on first use.

Everything works headless — no window activation, no mouse, no focus steal:

```bash
agent-cu snapshot -a Slack -i -c           # full DOM tree via CDP
agent-cu click @e5                         # JS element.click()
agent-cu key cmd+k -a Slack                # CDP key dispatch
agent-cu type "hello" -a Slack             # CDP insertText
agent-cu scroll down -a Slack              # JS scrollBy()
agent-cu text -a Slack                     # document.body.innerText
```

**Typing in Electron apps**: `insertText` goes to the focused element. If you need to type into a specific input:

```bash
agent-cu snapshot -a Slack -i -c           # find the input ref
agent-cu click @e18                        # click to focus it
agent-cu key cmd+a -a Slack                # select all
agent-cu key backspace -a Slack            # clear
agent-cu type "your text" -a Slack         # now type
```

## Verification

Never assume an action worked. Verify by checking a **state-bearing attribute**, not just by looking at the tree again.

### The `id` vs `name` distinction (critical)

Many apps give a button a **fixed `id`** (the slot) and a **changing `name`** (the current label).

Music's transport button is the canonical example:

- `id` is always `"play"` — it identifies the button as "the transport button", even when currently playing.
- `name` flips between `"play"` and `"pause"` depending on playback state.

**To detect state, read `name`, not `id`:**

```bash
# check if music is playing
agent-cu find 'id="play"' -a Music --compact
# → [{"name":"pause", ...}]   ← means: playback is ON
# → [{"name":"play", ...}]    ← means: playback is OFF
```

The same pattern appears in many apps: bookmark/unbookmark, mute/unmute, expand/collapse, follow/unfollow. When you want to confirm a toggle worked, always read the element's **current `name`** after the action.

### Inline verification with `--expect`

```bash
agent-cu click @e5 --expect 'name="Dashboard"'
# clicks, then polls for an element with name="Dashboard". Fails if it never appears.
```

### Reading values

```bash
agent-cu get-value @e3                     # one element's value + role + position
agent-cu find 'id="play"' -a Music --compact   # most stable if id is known
agent-cu snapshot -a Safari -i -c          # broad check
```

### Idempotent typing

```bash
agent-cu ensure-text @e3 "hello"           # only types if value differs
```

### Reading dynamic computed values (e.g., Calculator result)

Some apps don't surface the result as a normal `value` on a labeled element — it's hidden in a `staticText` node. Use `tree` and walk for any node with a `value`:

```bash
agent-cu tree -a Calculator -d 8 --compact | python3 -c "
import json, sys
d = json.load(sys.stdin)
def walk(n):
    if n.get('value'): print(n.get('role'), '=', repr(n['value']))
    for c in n.get('children', []): walk(c)
walk(d)
"
# → staticText = '1,234×7'
# → staticText = '8,638'
```

**Locale gotcha:** numbers are locale-formatted. Indian locale shows `7^8 = 57,64,801`, international shows `5,764,801`. They're the same value. Before comparing, strip commas and spaces.

## Waiting

When UI takes time to load:

```bash
agent-cu wait-for 'name="Dashboard"'       # poll until element appears
agent-cu wait-for 'role=button' --timeout 15
sleep 2                                # simple delay after navigation
```

## Batch operations

Chain multiple commands to avoid per-command startup:

```bash
echo '[["click","@e5"],["key","Return","-a","Music"]]' | agent-cu batch --bail
```

## Real-world patterns

### Search and play a song in Music (verified flow)

```bash
# 1. Open and snapshot
agent-cu open Music --wait
agent-cu snapshot -a Music -i -c
# → @e1 is the Search sidebar item

# 2. Click Search
agent-cu click @e1 -a Music

# 3. Type into the search field — use role=textField, not a ref (the ref
#    for the search field changes as the view switches)
agent-cu type "Espresso Sabrina Carpenter" -s 'role=textField' -a Music --submit
sleep 2  # let search results populate

# 4. Pick a result. Grep the snapshot for items matching the track name —
#    the `id` embeds a stable catalog id, so grab that.
agent-cu snapshot -a Music -c | grep -i "espresso" | head -5
# → [@e53] other("axcell") "Espresso" id=Music.shelfItem.TopSearchLockup[id=top-search-section-top-1744253558,...]

# 5. Open the album (double-click). Use the full id string, not the ref —
#    refs can drift between snapshots during long flows.
agent-cu click 'id="Music.shelfItem.TopSearchLockup[id=top-search-section-top-1744253558,parentId=top-search-section-top]"' -a Music --count 2
sleep 2

# 6. Find the track row and play. The track has a stable id pattern.
agent-cu snapshot -a Music -c | grep "track-lockup" | head -3
# → [@e52] group "Espresso" id=Music.shelfItem.AlbumTrackLockup[...]

# 7. Select, then try Return to play. If that fails, fall back to the
#    transport play button directly.
agent-cu click @e52 -a Music
agent-cu key Return -a Music
sleep 1

# 8. Verify via the transport button's `name` — "pause" means playing.
agent-cu find 'id="play"' -a Music --compact
# if name="play", playback didn't start. Fallback:
agent-cu click 'id="play"' -a Music
```

**Key lessons from this flow:**

- Refs (`@e53`) can go stale between snapshots separated by major UI changes. Prefer full `id="..."` for cross-snapshot targeting.
- `type -s 'role=textField' --submit` combines typing, clearing, and pressing Return reliably.
- Double-click on a search result often _opens_ the item, not _plays_ it. Drill into the detail view, then trigger playback.
- Always verify playback via the `name` of the transport button (`id="play"` is the slot; `name` holds state).
- If `Return` doesn't trigger playback, click the transport play button as a fallback. Have a plan B.

### Compute a multi-step calculation in Calculator (verified flow)

```bash
# 1. Open and snapshot — Calculator buttons have stable ids (Seven, Multiply, Equals, etc.)
agent-cu open Calculator --wait
agent-cu snapshot -a Calculator -i -c

# 2. Use batch for the whole keystroke sequence. Avoids 17 per-process starts.
#    Example: 7^8 = 7×7×7×7×7×7×7×7 = 5,764,801
echo '[
  ["click","id=\"AllClear\"","-a","Calculator"],
  ["click","id=\"Seven\"","-a","Calculator"],
  ["click","id=\"Multiply\"","-a","Calculator"],
  ["click","id=\"Seven\"","-a","Calculator"],
  ["click","id=\"Multiply\"","-a","Calculator"],
  ["click","id=\"Seven\"","-a","Calculator"],
  ["click","id=\"Multiply\"","-a","Calculator"],
  ["click","id=\"Seven\"","-a","Calculator"],
  ["click","id=\"Multiply\"","-a","Calculator"],
  ["click","id=\"Seven\"","-a","Calculator"],
  ["click","id=\"Multiply\"","-a","Calculator"],
  ["click","id=\"Seven\"","-a","Calculator"],
  ["click","id=\"Multiply\"","-a","Calculator"],
  ["click","id=\"Seven\"","-a","Calculator"],
  ["click","id=\"Multiply\"","-a","Calculator"],
  ["click","id=\"Seven\"","-a","Calculator"],
  ["click","id=\"Equals\"","-a","Calculator"]
]' | agent-cu batch

# 3. Read the result — walk the tree for any staticText with a value.
agent-cu tree -a Calculator -d 8 --compact | python3 -c "
import json, sys
d = json.load(sys.stdin)
for n in [d]:
    def walk(n):
        if n.get('value'): print(n.get('role'), '=', repr(n['value']))
        for c in n.get('children', []): walk(c)
    walk(n)
"
# → staticText = '7×7×7×7×7×7×7×7'     ← expression
# → staticText = '57,64,801'             ← result (Indian locale; strip commas → 5764801)
```

**Key lessons:**

- When an app exposes stable `id=` on every interactive element (Calculator does), skip snapshotting between clicks — just batch them.
- The result isn't on a labeled element; walk the tree for nodes with a `value`.
- Numbers come locale-formatted. Strip commas/spaces before parsing.

### Open a DM in Slack and send a message

```bash
agent-cu key cmd+k -a Slack
sleep 1
agent-cu snapshot -a Slack -i -c
# find and click the search input
agent-cu click @e18
agent-cu key cmd+a -a Slack
agent-cu key backspace -a Slack
agent-cu type "Vukasin" -a Slack
sleep 1
agent-cu key Return -a Slack
sleep 2
agent-cu type "hey, check this out" -a Slack
agent-cu key Return -a Slack
```

### Check calendar events

```bash
agent-cu open Calendar --wait
agent-cu snapshot -a Calendar -i -c -d 6
# read the visible dates
agent-cu text -a Calendar
# navigate to next month
agent-cu click @e3                         # next month button
sleep 1
agent-cu text -a Calendar
```

### Fill a web form

```bash
agent-cu open Safari --wait
agent-cu snapshot -a Safari -i -c
agent-cu type "https://example.com/form" -s @e34
agent-cu key Return -a Safari
sleep 3
agent-cu snapshot -a Safari -i -c -d 8
agent-cu type "John Doe" -s @e5
agent-cu type "john@example.com" -s @e6
agent-cu type "Hello world" -s @e7
agent-cu click @e8                         # submit button
agent-cu snapshot -a Safari -i -c          # verify submission
```

### Drag a file between Finder windows

```bash
# set up side-by-side windows
agent-cu move-window -a Finder --x 0 --y 25
# (ensure two windows open, Downloads left, Desktop right)
agent-cu snapshot -a Finder -i -c -d 8
# find the file (look for textfield with val="filename")
agent-cu click @e32                        # select the file
agent-cu drag --from-x 300 --from-y 55 --to-x 1000 --to-y 200 -a Finder
```

### Browse App Store

```bash
agent-cu open "App Store" --wait
agent-cu snapshot -a "App Store" -i -c -d 10
agent-cu click 'id="AppStore.tabBar.discover"' -a "App Store"
sleep 2
agent-cu scroll down --amount 5 -a "App Store"
agent-cu snapshot -a "App Store" -i -c -d 10
agent-cu text -a "App Store"
```

## Rules

1. **Always snapshot before acting.** You cannot interact with what you cannot see.
2. **Always re-snapshot after acting.** The UI changed. Your refs are stale.
3. **Refs for the next step, IDs for the long haul.** Refs (`@e5`) are fastest for the immediate next action after a snapshot. Full `id="..."` selectors survive across snapshots and are the right choice for anything you'll return to after other UI changes.
4. **Use `-i -c` on snapshots.** Interactive + compact reduces noise by 10x.
5. **Prefer `id=` over `name=` when the app provides stable ids.** IDs don't change with state; names often do.
6. **For state detection, read the `name` attribute — not `id`.** Many apps keep `id` as a fixed slot and flip `name` to reflect current state (play/pause, mute/unmute, bookmark/unbookmark).
7. **Wait after navigation.** `wait-for` a known landmark first; fall back to `sleep 2-3` if there's none.
8. **One action, then verify.** Don't chain multiple mutations without checking state between them.
9. **Use `type -s @ref`** over `type -a App`. The selector path uses AXSetValue (reliable). The app path uses keyboard simulation (fragile, needs focus).
10. **Use `scroll-to @ref`** when you know the element. It's headless. `scroll down` needs focus.
11. **Never hardcode app-specific logic.** Don't write a helper like `play_song_in_music()`. Use the same primitives (`snapshot`, `click`, `type`, `key`, `find`) for every app. The scene tree tells you what to do — read it, act on it, verify.
12. **Have a fallback for every action.** If a `click` doesn't produce the expected state change, escalate: double-click → scroll-to + click → keyboard shortcut → direct action button (e.g., transport `id="play"` instead of track-row Return). See the Recovery playbook.
13. **Batch when ids are stable and numerous.** Calculator, forms, keypad flows benefit from piping a JSON array into `agent-cu batch` — one process start instead of N.

## Recovery playbook

When something doesn't work, the principle is: **diagnose the exact failure mode, then apply the matching fallback**. Never retry the same command blindly.

### Element not found (stale ref)

Symptom: `error: element not found matching SelectorChain { ... }` on a `@ref` that came from an earlier snapshot.

Cause: refs resolve to a specific tree path; major UI changes (navigation, scroll, modal open) invalidate them.

Fix: re-snapshot and use the new ref, or switch to a stable `id=`:

```bash
agent-cu snapshot -a Safari -i -c
# …find it again, or use the full id you saw earlier:
agent-cu click 'id="the-stable-id"' -a Safari
```

Prefer `id="..."` for any target you'll act on across multiple snapshots. Refs are for single-step actions only.

### Ambiguous selector

Symptom: `error: ambiguous selector: found N elements matching`.

Fix: add `index=` or narrow by role/name:

```bash
agent-cu click 'role=button index=0' -a Music
agent-cu click 'role=button name="Save"' -a TextEdit
```

### Click succeeded but nothing happened

Symptom: `{"success": true, "message": "clicked at (X,Y)"}` but the expected state change (new page, playback, menu) did not occur.

Diagnose in order:

1. **Was AXPress used or did it fall back to coordinates?**
   - `pressed "Name" at (X,Y)` = AXPress succeeded (best).
   - `clicked at (X,Y)` = coordinate click; the element might have moved or be obscured.
2. **Is the element's state actually what you think?** Read the `name` attribute of the state-bearing element (e.g., Music's transport button), not just the tree.
3. **Does the action require focus?** Some apps (especially web UI) need the app to be frontmost. `agent-cu open App --wait` activates it.

Fallback ladder:

```bash
# Ladder 1: AXPress failed — try double-click
agent-cu click @e5 --count 2

# Ladder 2: Element offscreen — bring it in, then click
agent-cu scroll-to @e5
agent-cu click @e5

# Ladder 3: The clicked element isn't the one that triggers the action —
#   drill into the container and find the explicit action button
agent-cu snapshot -a Music -c | grep -E "play|pause|Next"
agent-cu click 'id="play"' -a Music

# Ladder 4: Keyboard shortcut fallback
agent-cu key Return -a Music          # activate selected item
agent-cu key space -a Music           # play/pause
```

### Type didn't land

Symptom: text didn't appear in the field.

Causes + fixes:

```bash
# Cause: no ref — fell back to keyboard sim which needs app focus
agent-cu type "hello" -a Safari
# Fix: use selector path (AXSetValue, reliable)
agent-cu type "hello" -s @e3

# Cause: field wasn't focused when keyboard sim fired
agent-cu click @e3              # focus it first
agent-cu type "hello" -s @e3    # then type

# Cause: existing value not cleared
agent-cu type "new" -s @e3 --append   # wrong: keeps old text
agent-cu type "new" -s @e3            # right: clears first
```

### Search submitted but results aren't in the tree yet

Symptom: `snapshot` right after `--submit` shows the old page.

Fix: wait, then re-snapshot. Prefer polling over `sleep`:

```bash
agent-cu type "query" -s 'role=textField' -a Music --submit
agent-cu wait-for 'name~="Top Results"' --timeout 5
agent-cu snapshot -a Music -i -c
```

If `wait-for` doesn't know a stable landmark, fall back to `sleep 2`.

### Snapshot is too shallow — can't find the element

Bump depth:

```bash
agent-cu snapshot -a Safari -i -c -d 8
agent-cu snapshot -a Safari -i -c -d 12   # for deeply nested apps
```

### Ref points to the wrong thing

Symptom: `@e5` clicks something but it's not what you expected.

Cause: the snapshot re-ran silently (you took two) and refs got renumbered.

Fix: always inspect the ref's resolved element before acting on anything important:

```bash
agent-cu get-value @e5 -a Safari
# confirm role, name, id are what you expect, THEN click
```

### Electron app not using CDP

agent-cu auto-detects Electron apps. If CDP isn't working:

```bash
agent-cu snapshot -a Slack -i -c -v        # verbose shows CDP status
```

First run takes ~5s as the app relaunches with `--remote-debugging-port`. Every subsequent run uses a cached connection (~15ms).

### DSL selector parse error

Symptom: `unknown filter key: 'id_contains'`.

Fix: the syntax is `id~=` (tilde-equals) for partial match, not `id_contains=`:

```bash
agent-cu click 'id~="track-"' -a Music
agent-cu click 'name~="Submit"' -a Safari
```

## Output format

All output is JSON:

```json
{"success": true, "message": "pressed \"7\" at (453, 354)"}
{"error": true, "type": "element_not_found", "message": "..."}
{"role": "button", "name": "Submit", "value": null, "position": {"x": 450, "y": 320}}
```
