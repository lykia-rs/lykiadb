var $q = SELECT * FROM extremely_good_songs
UNION
SELECT * FROM extremely_bad_songs
INTERSECT
SELECT * FROM paid_songs
EXCEPT
SELECT * FROM premium_songs;

print($q);