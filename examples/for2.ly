var $loopStart = clock();

for (var $i = 0; $i < 10; $i = $i + 1) {
    if ($i == 2) continue;
    if ($i == 8) break;
    print($i);
}
var $loopTime = clock() - $loopStart;

print("loop:", $loopTime);