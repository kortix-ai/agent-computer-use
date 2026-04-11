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

Never assume an action worked. Always verify:

```bash
# After clicking:
agent-cu snapshot -a Safari -i -c          # check what changed

# After typing:
agent-cu get-value @e3                     # verify the value

# Inline verification:
agent-cu click @e5 --expect 'name="Done"'  # click then wait for "Done"

# Idempotent typing:
agent-cu ensure-text @e3 "hello"           # only types if value differs
```

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

### Search and play a song

```bash
agent-cu open Music --wait
agent-cu snapshot -a Music -i -c
# find search — click it
agent-cu click @e1
agent-cu snapshot -a Music -i -c
# find search field
agent-cu type "Kiss of Life" -s @e31
agent-cu key Return -a Music
sleep 3
agent-cu snapshot -a Music -i -c -d 8
# find the track, double-click to play
agent-cu click 'id~="604771089"' -a Music --count 2
sleep 2
agent-cu get-value 'id="title"' -a Music
# → "Kiss of Life"
```

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
3. **Use refs, not selectors.** Refs are fast and unambiguous. Selectors search the tree.
4. **Use `-i -c` on snapshots.** Interactive + compact reduces noise by 10x.
5. **Use `id=` selectors when available.** IDs are the most stable across UI changes.
6. **Wait after navigation.** `sleep 2-3` after opening pages, switching tabs, submitting forms.
7. **Verify after typing.** Use `get-value` to confirm the text was set correctly.
8. **One action at a time.** Don't chain multiple actions without checking state between them.
9. **Use `type -s @ref`** over `type -a App`. The selector path uses AXSetValue (reliable). The app path uses keyboard simulation (fragile).
10. **Use `scroll-to @ref`** when you know the element. It's headless. `scroll down` needs focus.

## Troubleshooting

### Element not found

Re-snapshot. Your refs are stale.

```bash
agent-cu snapshot -a Safari -i -c
agent-cu click @e3                         # use the NEW ref
```

### Ambiguous selector

Multiple elements match. Use refs instead, or add `index=`:

```bash
agent-cu click 'role=button index=0' -a Music
```

### Click didn't work

Try double-click (some apps need it to activate items):

```bash
agent-cu click @e5 --count 2
```

Or use `scroll-to` first if the element might be offscreen:

```bash
agent-cu scroll-to @e5
agent-cu click @e5
```

### Type didn't work

Use the selector path, not the app path:

```bash
# wrong (keyboard sim, fragile):
agent-cu type "hello" -a Safari

# right (AXSetValue, reliable):
agent-cu type "hello" -s @e3
```

### Electron app not using CDP

agent-cu auto-detects Electron apps. If it's not working:

```bash
agent-cu snapshot -a Slack -i -c -v        # verbose shows CDP status
```

If the app wasn't auto-relaunched, it will be on next run. First run takes ~5s.

## Output format

All output is JSON:

```json
{"success": true, "message": "pressed \"7\" at (453, 354)"}
{"error": true, "type": "element_not_found", "message": "..."}
{"role": "button", "name": "Submit", "value": null, "position": {"x": 450, "y": 320}}
```
