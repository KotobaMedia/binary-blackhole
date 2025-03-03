# DB Setup

Right now, we use a Postgres database. In CDK, we set up RDS. RDS limits creating extensions from superuser accounts, so use the rds_admin account that RDS has set up for you to create the database (default `bbh`) and run `CREATE EXTENSION "postgis";`.

We use 2 roles: `bbh_admin` and `bbh_ro`. The app uses ro, it only has select permissions. admin is used for tools to load SQL data in.

```sql
create user bbh_admin with password '...';
grant all privileges on database bbh to bbh_admin;

create user bbh_ro with password '...';
grant usage on schema public to bbh_ro;
grant select on all tables in schema public to bbh_ro;
alter default privileges in schema public grant select on tables to bbh_ro;
```
