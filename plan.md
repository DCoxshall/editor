TODO: Fix VisualBox::draw's idiotic line_col argument.

> User will input command, either via the command prompt (with all arguments
included) or with a keypress (which then prompts the user for the arguments).

> When the user inputs a command via the terminal, the system attempts to run
the command. Failure is then reported back to the user.

## Commands

- next_tab

- previous_tab

- focus_tab tab_idx

- (m)ove_focus [(l)eft|(r)ight|(u)p|(d)own]

- quit

- close

Like `quit`, but only closes a single file.

- save [filename] [!]

If `filename` is present, create/overwrite the corresponding file and fill it with the contents of the currently focused view. Overwriting should fail with a warning if the given file path does not match the path in the currently focused view, unless the command ends in '!'.

If `filename` is not present, save to the filepath of the currently focused view. If the file does not exist, fail with a warning.

- open filename

Opens the given file in a new tab, and switches the focus to this new tab.

- vsplit filename [l|r]

Opens the given file in a vertical split - that is, one file next to the other. The opened view will open to the (l)eft or (r)ight of the current window. Default is right.

- hsplit filename [u|d]

Opens the given file in a horizontal split - that is, one file above the other. The opened view will open above (u) or below (d) of the current window. Default is below.
