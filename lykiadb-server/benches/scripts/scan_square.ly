var $arr = arr::new(100);

var $result = select 
                item * item as p, 
                item * item as q,
                item * item as r
            from $arr as item;