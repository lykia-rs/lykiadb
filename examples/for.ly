for (var $i = 0; $i < 10000000; $i = $i+1) {
    if ($i > 20) break;
    if ($i < 10) continue;
    for (var $j = 0; $j < 10000000; $j = $j + 1) {
        print("" + $j + ", " + $i);
        if ($j > 3) break;
    }
}