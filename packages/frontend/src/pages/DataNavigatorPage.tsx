import React, { useState, useEffect } from "react";
import useSWR from "swr";
import clsx from "clsx";
import { fetcher } from "../tools/api";
import {
  CaretDownFill,
  CaretRightFill,
  Table as TableIcon,
} from "react-bootstrap-icons";

// Types for API response
interface Column {
  name: string;
  desc?: string;
  data_type: string;
  enum_values?: { value: string; desc: string }[];
  foreign_key?: { foreign_table: string; foreign_column: string };
  primary_key?: boolean;
}

interface Table {
  table_name: string;
  name?: string;
  desc?: string;
  source?: string;
  source_url?: string;
  license?: string;
  primary_key?: string;
  columns: Column[];
}

interface TableListResponse {
  tables: Table[];
}

// Geometry type detection function
const getGeometryType = (table: Table): "point" | "line" | "polygon" | null => {
  // Look for geometry columns
  const geometryColumns = table.columns.filter((col) =>
    col.data_type.startsWith("geometry("),
  );

  if (geometryColumns.length === 0) {
    return null;
  }

  // Extract geometry type from the first geometry column
  const geometryType = geometryColumns[0].data_type;

  // Parse the geometry type from format like "geometry(MULTIPOLYGON, 6668)"
  const match = geometryType.match(/geometry\(([^,]+)/i);
  if (!match) {
    return null;
  }

  const typeName = match[1].toLowerCase();

  // Map PostGIS geometry types to our simplified types
  if (typeName.includes("point")) {
    return "point";
  } else if (typeName.includes("line") || typeName.includes("linestring")) {
    return "line";
  } else if (typeName.includes("polygon")) {
    return "polygon";
  }

  return null;
};

// Geometry icon component
const GeometryIcon: React.FC<{
  geometryType: "point" | "line" | "polygon" | null;
}> = ({ geometryType }) => {
  if (geometryType === null) {
    return (
      <TableIcon
        className="text-body me-2"
        title="位置情報を持たないテーブル"
      />
    );
  }

  const getTitle = () => {
    switch (geometryType) {
      case "point":
        return "点 (Point)";
      case "line":
        return "線 (Line)";
      case "polygon":
        return "面 (Polygon)";
    }
  };

  let iconName = null;
  switch (geometryType) {
    case "point":
      iconName = "fg-point";
      break;
    case "line":
      iconName = "fg-polyline-pt";
      break;
    case "polygon":
      iconName = "fg-polygon-pt";
      break;
  }
  return (
    <span className={clsx(iconName, "text-body", "me-2")} title={getTitle()} />
  );
};

// Column table component
interface ColumnTableProps {
  columns: Column[];
}

const ColumnTable: React.FC<ColumnTableProps> = ({ columns }) => {
  return (
    <div className="ps-4 pe-2 pb-2">
      <div className="table-responsive">
        <table className="table table-sm table-bordered table-hover my-0">
          <thead className="table">
            <tr>
              <th scope="col" style={{ width: "25%" }}>
                カラム名
              </th>
              <th scope="col" style={{ width: "20%" }}>
                データ型
              </th>
              <th scope="col" style={{ width: "35%" }}>
                説明
              </th>
              <th scope="col" style={{ width: "20%" }}>
                詳細
              </th>
            </tr>
          </thead>
          <tbody>
            {columns.map((column) => (
              <tr key={column.name}>
                <td>
                  <code className="text-primary text-body">
                    <strong>{column.name}</strong>
                  </code>
                  {column.primary_key && (
                    <span className="badge bg-primary ms-1">主キー</span>
                  )}
                </td>
                <td>
                  <span className="badge bg-secondary">{column.data_type}</span>
                </td>
                <td>
                  {column.desc ? (
                    <span className="text-muted">{column.desc}</span>
                  ) : (
                    <span className="text-muted fst-italic">説明なし</span>
                  )}
                </td>
                <td>
                  {column.foreign_key && (
                    <div className="small">
                      <span className="text-info">外部キー:</span>{" "}
                      {column.foreign_key.foreign_table}.
                      {column.foreign_key.foreign_column}
                    </div>
                  )}
                  {column.enum_values && column.enum_values.length > 0 && (
                    <div className="small">
                      <span className="text-info">列挙型:</span>{" "}
                      {column.enum_values.length}個の値
                      <table className="table table-sm table-borderless mt-1 mb-0">
                        <tbody>
                          {column.enum_values.map((enumVal, index) => (
                            <tr key={index} className="border-0">
                              <td
                                className="p-0 pe-2 text-muted"
                                style={{ fontSize: "0.75rem" }}
                              >
                                <code>{enumVal.value}</code>
                              </td>
                              <td
                                className="p-0 text-muted"
                                style={{ fontSize: "0.75rem" }}
                              >
                                {enumVal.desc}
                              </td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};

// Table expanded content component
interface TableExpandedContentProps {
  table: Table;
}

const TableExpandedContent: React.FC<TableExpandedContentProps> = ({
  table,
}) => {
  return (
    <div className="my-4">
      {/* Table metadata */}
      {(table.source || table.source_url || table.license) && (
        <div className="ps-4 pe-2 pb-2">
          <dl className="row mb-3">
            {table.source && (
              <>
                <dt className="col-sm-3 text-muted">出典:</dt>
                <dd className="col-sm-9">{table.source}</dd>
              </>
            )}
            {table.source_url && (
              <>
                <dt className="col-sm-3 text-muted">出典URL:</dt>
                <dd className="col-sm-9">
                  <a
                    href={table.source_url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-primary"
                  >
                    {table.source_url}
                  </a>
                </dd>
              </>
            )}
            {table.license && (
              <>
                <dt className="col-sm-3 text-muted">ライセンス:</dt>
                <dd className="col-sm-9">{table.license}</dd>
              </>
            )}
          </dl>
        </div>
      )}
      <ColumnTable columns={table.columns} />
    </div>
  );
};

// Table component
interface TableItemProps {
  table: Table;
  isExpanded: boolean;
  isSelected: boolean;
  onToggleExpand: (tableName: string) => void;
  onToggleSelection: (table: Table) => void;
}

const TableItem: React.FC<TableItemProps> = ({
  table,
  isExpanded,
  isSelected,
  onToggleExpand,
  onToggleSelection,
}) => {
  const geometryType = getGeometryType(table);

  return (
    <li className="list-group-item p-0">
      <div
        className="p-3"
        style={{ cursor: "pointer", display: "flex", alignItems: "center" }}
        onClick={(e) => {
          if ((e.target as HTMLElement).closest(".form-check-input")) return;
          onToggleExpand(table.table_name);
        }}
      >
        <div
          className="d-flex align-items-center mb-1 flex-grow-1"
          style={{ width: "100%" }}
        >
          <div className="form-check me-2">
            <input
              className="form-check-input"
              type="checkbox"
              id={`table-${table.table_name}`}
              checked={isSelected}
              onChange={() => onToggleSelection(table)}
              onClick={(e) => e.stopPropagation()}
            />
          </div>
          <button
            className="btn btn-sm me-2"
            onClick={(e) => {
              e.stopPropagation();
              onToggleExpand(table.table_name);
            }}
            aria-label={isExpanded ? "折りたたむ" : "展開"}
            tabIndex={-1}
          >
            {isExpanded ? <CaretDownFill /> : <CaretRightFill />}
          </button>
          <GeometryIcon geometryType={geometryType} />
          <span style={{ minWidth: 200, flex: 1, fontWeight: 500 }}>
            {table.name || table.table_name}
          </span>
          {table.desc && (
            <span className="text-muted ms-2" style={{ flex: 2 }}>
              {table.desc}
            </span>
          )}
          <span className="text-muted ms-2" style={{ whiteSpace: "nowrap" }}>
            [{table.columns.length}カラム]
          </span>
        </div>
      </div>
      {isExpanded && <TableExpandedContent table={table} />}
    </li>
  );
};

// Selected data component
interface SelectedDataProps {
  selectedTables: Table[];
  onProceed: () => void;
  onRemoveTable: (table: Table) => void;
  onClearAll: () => void;
}

const SelectedData: React.FC<SelectedDataProps> = ({
  selectedTables,
  onProceed,
  onRemoveTable,
  onClearAll,
}) => {
  return (
    <div className="card">
      <div className="card-body">
        <div className="d-flex justify-content-between align-items-center mb-3">
          <h5 className="card-title mb-0">選択されたテーブル</h5>
          {selectedTables.length > 0 && (
            <button
              className="btn btn-sm btn-outline-secondary"
              onClick={onClearAll}
            >
              すべてクリア
            </button>
          )}
        </div>
        {selectedTables.length === 0 ? (
          <div className="text-muted fst-italic my-3">
            テーブルが選択されていません。
          </div>
        ) : (
          <ul className="list-group mb-3">
            {selectedTables.map((table) => (
              <li
                key={table.table_name}
                className="list-group-item d-flex justify-content-between align-items-center"
              >
                <span>{table.name || table.table_name}</span>
                <button
                  className="btn btn-sm btn-outline-danger"
                  onClick={() => onRemoveTable(table)}
                  aria-label={`${table.name || table.table_name}を削除`}
                >
                  ×
                </button>
              </li>
            ))}
          </ul>
        )}
        <button
          className="btn btn-primary w-100"
          disabled={selectedTables.length === 0}
          onClick={onProceed}
        >
          進む
        </button>
      </div>
    </div>
  );
};

const GROUP_LABEL = "国土数値情報＆国勢調査";

const DataNavigatorPage: React.FC = () => {
  const { data, error, isLoading } = useSWR<TableListResponse>(
    "/datasets",
    fetcher,
  );
  const [expanded, setExpanded] = useState<{ [key: string]: boolean }>({});
  const [selectedTables, setSelectedTables] = useState<Table[]>([]);
  const [search, setSearch] = useState("");
  const [filteredTables, setFilteredTables] = useState<Table[]>([]);

  // When data loads, set default expanded state
  useEffect(() => {
    if (!data) return;
    const expanded: { [key: string]: boolean } = {};
    expanded[GROUP_LABEL] = true;
    data.tables.forEach((table) => {
      expanded[table.table_name] = false;
    });
    setExpanded(expanded);
    setFilteredTables(data.tables);
  }, [data]);

  // Filter logic
  useEffect(() => {
    if (!data) return;
    if (!search.trim()) {
      setFilteredTables(data.tables);
      return;
    }
    const q = search.trim().toLowerCase();
    const filtered = data.tables
      .map((table) => {
        if (
          table.table_name.toLowerCase().includes(q) ||
          (table.name && table.name.toLowerCase().includes(q))
        )
          return table;
        // Filter columns
        const columns = table.columns.filter(
          (col) =>
            col.name.toLowerCase().includes(q) ||
            (col.desc && col.desc.toLowerCase().includes(q)),
        );
        if (columns.length > 0) {
          return { ...table, columns };
        }
        return null;
      })
      .filter(Boolean) as Table[];
    setFilteredTables(filtered);
  }, [search, data]);

  const toggleExpand = (key: string) => {
    setExpanded((prev) => ({ ...prev, [key]: !prev[key] }));
  };

  const toggleTableSelection = (table: Table) => {
    setSelectedTables((prev) => {
      const isSelected = prev.some((t) => t.table_name === table.table_name);
      if (isSelected) {
        return prev.filter((t) => t.table_name !== table.table_name);
      } else {
        return [...prev, table];
      }
    });
  };

  const handleProceed = () => {
    const tableNames = selectedTables.map(
      (table) => table.name || table.table_name,
    );
    alert("ChatMapPageに進みます。テーブル: " + tableNames.join(", "));
  };

  const handleRemoveTable = (table: Table) => {
    setSelectedTables((prev) =>
      prev.filter((t) => t.table_name !== table.table_name),
    );
  };

  const handleClearAll = () => {
    setSelectedTables([]);
  };

  return (
    <div className="container-fluid py-4">
      <div className="row">
        <div className="col-9">
          <div className="card">
            <div className="card-body">
              <h5 className="card-title">データベース</h5>
              <div className="mb-3">
                <input
                  type="text"
                  className="form-control"
                  placeholder="テーブルまたはカラムを検索..."
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                />
              </div>
              {isLoading && <div className="text-muted">読み込み中...</div>}
              {error && (
                <div className="text-danger">
                  データの読み込みに失敗しました。
                </div>
              )}
              <ul className="list-group list-group-flush">
                {!isLoading && !error && filteredTables.length === 0 && (
                  <li className="list-group-item text-muted">
                    結果が見つかりません。
                  </li>
                )}
                <li key={GROUP_LABEL} className="list-group-item">
                  <div className="d-flex align-items-center mb-1">
                    <button
                      className="btn btn-sm btn-outline-secondary me-2"
                      onClick={() => toggleExpand(GROUP_LABEL)}
                      aria-label={expanded[GROUP_LABEL] ? "折りたたむ" : "展開"}
                    >
                      {expanded[GROUP_LABEL] ? "-" : "+"}
                    </button>
                    <span className="fw-bold">{GROUP_LABEL}</span>
                    <span className="text-muted ms-2">
                      [{filteredTables.length}テーブル]
                    </span>
                  </div>
                  {expanded[GROUP_LABEL] && (
                    <ul className="list-group list-group-flush ms-4">
                      {filteredTables.map((table) => (
                        <TableItem
                          key={table.table_name}
                          table={table}
                          isExpanded={expanded[table.table_name]}
                          isSelected={selectedTables.some(
                            (t) => t.table_name === table.table_name,
                          )}
                          onToggleExpand={toggleExpand}
                          onToggleSelection={toggleTableSelection}
                        />
                      ))}
                    </ul>
                  )}
                </li>
              </ul>
            </div>
          </div>
        </div>
        <div className="col-3">
          <div className="sticky-top" style={{ top: "1.5rem" }}>
            <SelectedData
              selectedTables={selectedTables}
              onProceed={handleProceed}
              onRemoveTable={handleRemoveTable}
              onClearAll={handleClearAll}
            />
          </div>
        </div>
      </div>
    </div>
  );
};

export default DataNavigatorPage;
