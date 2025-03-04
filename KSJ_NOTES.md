# KSJメモ

```sql
ALTER TABLE n03 RENAME COLUMN 群名 TO 郡名;

CREATE MATERIALIZED VIEW n03_union AS (
    SELECT
        ...,
        ST_Union(geom) AS geom
    FROM n03
    GROUP BY ...
);
```
