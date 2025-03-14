# KSJメモ

```sql
ALTER TABLE n03 RENAME COLUMN 群名 TO 郡名;

CREATE MATERIALIZED VIEW n03_union AS (
    SELECT
        n03.都道府県名, n03.北海道の振興局名, n03.郡名, n03.市区町村名, n03.政令指定都市の行政区域名, n03.全国地方公共団体コード,
        ST_Union(geom) AS geom
    FROM n03
    GROUP BY n03.都道府県名, n03.北海道の振興局名, n03.郡名, n03.市区町村名, n03.政令指定都市の行政区域名, n03.全国地方公共団体コード
);

UPDATE datasets SET table_name = 'n03_union' WHERE table_name = 'n03';
```
