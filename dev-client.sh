
SESSION="lykiadb-client-server"

# Kill existing session if it exists
tmux kill-session -t $SESSION 2>/dev/null || true

# Yeni bir tmux oturumu oluştur, ilk komutla server panelini başlat
tmux new-session -d -s $SESSION -n main "cargo run --bin lykiadb-server"

# New pane + client with auto-reload
tmux split-window -v -t $SESSION:0 "cargo watch -w lykiadb-shell/src -w lykiadb-lang/src -x 'run --bin lykiadb-shell'"

# Optional: Resize panes (e.g. make the top pane 15 lines high)
tmux resize-pane -t $SESSION:0.0 -y 15

# Attach to session
tmux attach -t $SESSION