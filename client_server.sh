
SESSION="lykiadb-client-server"

# Yeni bir tmux oturumu oluştur, ilk komutla server panelini başlat
tmux new-session -d -s $SESSION -n main "cargo run --bin lykiadb-server"

# Yeni bir panel oluştur ve client'ı başlat
tmux split-window -v -t $SESSION:0 "cargo run --bin lykiadb-shell"

# İsteğe bağlı: Panelleri yeniden boyutlandır (örneğin üst panel 15 satır)
tmux resize-pane -t $SESSION:0.0 -y 15

# Oturuma bağlan
tmux attach -t $SESSION