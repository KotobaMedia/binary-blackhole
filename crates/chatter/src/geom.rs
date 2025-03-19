use geo_traits::to_geo::ToGeoGeometry;
use geo_types::Geometry;
use tokio_postgres::types::{FromSql, Type};

// This handles converting PostGIS geometry to geo_types::Geometry
#[derive(Debug, PartialEq)]
pub struct GeometryWrapper(pub Geometry<f64>);

impl<'a> FromSql<'a> for GeometryWrapper {
    fn from_sql(
        _ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let wkb_geom = wkb::reader::read_wkb(raw)?;
        let geometry = wkb_geom.try_to_geometry().ok_or_else(|| {
            Box::<dyn std::error::Error + Sync + Send>::from("Invalid WKB geometry")
        })?;

        Ok(GeometryWrapper(geometry))
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "geometry" || ty.name() == "geography"
    }
}

impl GeometryWrapper {
    pub fn variant_name(&self) -> &'static str {
        match self.0 {
            Geometry::Point(_) => "Point",
            Geometry::LineString(_) => "LineString",
            Geometry::Polygon(_) => "Polygon",
            Geometry::MultiPoint(_) => "MultiPoint",
            Geometry::MultiLineString(_) => "MultiLineString",
            Geometry::MultiPolygon(_) => "MultiPolygon",
            Geometry::GeometryCollection(_) => "GeometryCollection",
            Geometry::Line(_) => "Line",
            Geometry::Rect(_) => "Rect",
            Geometry::Triangle(_) => "Triangle",
        }
    }
}

impl ToString for GeometryWrapper {
    fn to_string(&self) -> String {
        self.variant_name().to_string()
    }
}
