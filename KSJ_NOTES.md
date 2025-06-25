# KSJメモ

```sql
ALTER TABLE n03 RENAME COLUMN 群名 TO 郡名;

CREATE MATERIALIZED VIEW n03_union AS (
    SELECT
        n03.都道府県名,
        n03.北海道の振興局名,
        n03.郡名,
        n03.市区町村名,
        n03.政令指定都市の行政区域名,
        n03.全国地方公共団体コード,
        ST_Multi(ST_Union(geom))::geometry(MultiPolygon,6668) AS geom
    FROM n03
    GROUP BY n03.都道府県名, n03.北海道の振興局名, n03.郡名, n03.市区町村名, n03.政令指定都市の行政区域名, n03.全国地方公共団体コード
);
CREATE INDEX idx_n03_union_geom ON n03_union USING GIST (geom);

UPDATE datasets SET table_name = 'n03_union' WHERE table_name = 'n03';
```

## 未設定SRID修正

_todo: jpksj-to-sqlレイヤーで修正する_

SRID=0を列挙する

```sql
SELECT
  f_table_schema   AS schema_name,
  f_table_name     AS table_name,
  f_geometry_column AS column_name,
  srid,
  type            AS geom_base_type,
  format(
    'geometry(%s,%s)',
    type,
    srid
  )               AS column_type
FROM geometry_columns g
WHERE g.srid = 0
ORDER BY schema_name, table_name, column_name;
```

修正する

```sql
ALTER TABLE [table name here]
  ALTER COLUMN geom
  TYPE geometry(MultiPolygon, 6668)
  USING ST_SetSRID(geom, 6668);
```
