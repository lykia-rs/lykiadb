var $arr = arr::new(100);
var $result = 
    select 
        col_0 as mod,
        avg(item * item * item) as avg,
        count(1) as ct
    from $arr as item 
    group by mod(item, 3);