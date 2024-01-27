var $i = 5;

var $p = SELECT *, $i as five FROM some_collection
         UNION
         SELECT *, 6 as six FROM some_other_collection;

var $q = SELECT * FROM users where id != 5;

var $r = SELECT "darkness" as my_old_friend;

print($p);
print($q);
print($r);