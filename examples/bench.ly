fun heavyop() {
    for (var $i = 0; $i < 1000; $i = $i+1) {}
}
print(bench(heavyop, 100));