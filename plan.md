> User will input command, either via the command prompt (with all arguments
included) or with a keypress (which then prompts the user for the arguments).

> When the user inputs a command via the terminal, the system attempts to run
the command. Failure is then reported back to the user.

## Commands

- next_tab

- previous_tab

- move_focus_left

- move_focus_right

- move_focus_up

- move_focus_down

- quit

- save [filename]

If `filename` is present, create/overwrite the corresponding file and fill it with the contents of the currently focused view. Overwriting should fail with a warning if the given file path does not match the path in the currently focused view, unless the command ends in '!'.

If `filename` is not present, save to the filepath of the currently focused view. If the file does not exist, fail with a warning.

- open filename

- vsplit filename

- hsplit filename