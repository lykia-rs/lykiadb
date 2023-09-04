var $i = 5;
var $q = 
SELECT *, $i as five FROM extremely_good_songs
UNION ALL
SELECT *, 6 as six FROM extremely_bad_songs;

print($q);