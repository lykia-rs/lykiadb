
SESSION="lykiadb-client-server"

tmux kill-session -t $SESSION 2>/dev/null || true

tmux new-session -d -s $SESSION -n main "cargo run --bin lykiadb-server"

tmux split-window -v -t $SESSION:0 "cargo watch -w lykiadb-shell/src -w lykiadb-lang/src -x 'run --bin lykiadb-shell'"

tmux resize-pane -t $SESSION:0.0 -y 15

tmux attach -t $SESSION