# Five mockups of the screen, of 2026-08-16

**The maintainer asked for five mockups of the interface, and they decide which
of them the program takes.** Each of the five comes of a different lineage of a
TUI that many people like, and each of them answers the same four items of
`### 0. The four items of the maintainer of 2026-08-16` of `docs/HANDOVER.md`.

**Every file `mockup-N.txt` is a real grid of characters of 160 columns and 45
rows**, therefore a terminal can draw every one of them. The file
`mockup-N.md` beside it holds the lineage, a walk through each region, the
answer of each of the four items, a table of the colours (the 16 names of ANSI
and the default foreground and background alone), the map of the mouse, and
what that design gives up.

| The file | The lineage | The one idea |
|---|---|---|
| `mockup-1` | lazygit, gitui, tig | Seven panels, and the marks of the mouse stand in the titles of the panels: `◉` for the focus, `⇕` for the wheel, `↔` for a drag. The focus is the weight of the border and not a colour. |
| `mockup-2` | yazi, ranger, nnn, lf | A series is the next column, and the rows that the short list of a series leaves become a grid of covers. One rule answers the series and the empty space together. |
| `mockup-3` | ncspot, musikcube, cmus | Two bars of the position, one inside the chapter and one across the whole book, with a row of marks `╫` of the chapters under the second one. Each mark is a target of a click. |
| `mockup-4` | k9s, btop, lazydocker | A table of columns that a click sorts, chips of the filters, and a line of commands (`:sort title`). A rail of 43 columns holds one large cover and a strip of six. |
| `mockup-5` | television, fzf, superfile | The library **is** the grid of the covers, and a line of a query narrows it while the user writes. 8 tiles of 18 cells at 160 columns, 5 at 80, and a list under 56. |

**A page that holds the five screens side by side, and the table of the
comparison**, stands at
<https://claude.ai/code/artifact/361f7100-f479-4ccc-9cd5-47236d845ebf>.

## The decision

**The maintainer chose the `mockup-1`, the panels, on 2026-08-16.** That file
and its note are the design of the program now, and the four items of the
maintainer are the parts of it. `### 0. The road of the panels (T-316 to
T-323)` of `docs/HANDOVER.md` holds the eight stages of the work in their
sequence, and the three decisions that a round must not take alone.

The other four mockups stay in this directory. They hold ideas that the road
can take later, and each of them says what it gives up: the two bars of the
position of `mockup-3`, the rail of the covers of `mockup-4`, the series of a
column of `mockup-2`, and the query that narrows a grid of `mockup-5`.

## The two mockups of 2026-08-16, of the second report of the maintainer

**The maintainer read the program v0.8.158 and they asked for two views of a
new shape.** These two mockups keep the panels of `mockup-1` and they change
the view inside the panel 4 alone.

| The file | The view | The one idea |
|---|---|---|
| `mockup-6` | Home | Every shelf of the server is a band of covers, with `j`/`k` for a shelf and `h`/`l` for a cover. The table of 55 rows goes away. |
| `mockup-7` | Chapters | The two bars of `mockup-3` over a table of the number, the title, the start, and the length. `Enter` plays a chapter. |

`docs/TAKEOVER-BACKLOG.md` holds the work of the two of them: T-330 is the
sweep of the words and the pictures, which holds the Chapters view of
`mockup-7`, and T-331 is the new Home view of `mockup-6`.
