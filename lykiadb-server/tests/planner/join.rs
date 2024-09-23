use lykiadb_server::{assert_plan, plan::planner::test_helpers::expect_plan};


assert_plan! {
    three_way_simple: {
        "SELECT * FROM books b INNER JOIN categories c ON b.category_id = c.id 
        INNER JOIN publishers AS p ON b.publisher_id = p.id where p.name = 'Springer';" => 

"- filter (p.name IsEqual Str(\"Springer\")):
  - join [Inner, (b.publisher_id IsEqual p.id)]:
    - join [Inner, (b.category_id IsEqual c.id)]:
      - scan [books as b]
      - scan [categories as c]
    - scan [publishers as p]
"
    }
}
