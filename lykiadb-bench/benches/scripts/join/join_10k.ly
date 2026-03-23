var $left = arr::new(100);
var $right = arr::new(100);
var $q = select l, r from
            (
              $left AS l
              cross join
              $right AS r
            );