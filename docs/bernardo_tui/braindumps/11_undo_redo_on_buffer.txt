There is a decision to make, what is the minimal change that justifies having a undo-option. Now I just do it every char.
That's probably overkill.

I tried something smart, but I failed two approaches and I'm done for now. I guess best way is to "create history" and
then possibly "compress history", to merge multiple edits tailing the history as single larger chunk. But now, I leave
it as it is.

also, look at comment in apply_cem in buffer_state