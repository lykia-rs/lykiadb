fun f($x, $q) {
    $x($q);
}

fun g($q) {
    print($q);
}

for (var $i=0; $i<10; $i = $i + 1) {
    f(g, $i);
}
