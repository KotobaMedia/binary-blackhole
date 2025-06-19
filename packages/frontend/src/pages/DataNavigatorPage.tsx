import React, { useState, useEffect } from "react";
import useSWR from "swr";
import { fetcher } from "../tools/api";

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

// Column table component
interface ColumnTableProps {
  columns: Column[];
}

const ColumnTable: React.FC<ColumnTableProps> = ({ columns }) => {
  return (
    <div className="ps-4 pe-2 pb-2">
      <div className="table-responsive">
        <table className="table table-sm table-bordered table-hover">
          <thead className="table">
            <tr>
              <th scope="col" style={{ width: "25%" }}>
                Column Name
              </th>
              <th scope="col" style={{ width: "20%" }}>
                Data Type
              </th>
              <th scope="col" style={{ width: "35%" }}>
                Description
              </th>
              <th scope="col" style={{ width: "20%" }}>
                Details
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
                    <span className="badge bg-primary ms-1">PK</span>
                  )}
                </td>
                <td>
                  <span className="badge bg-secondary">{column.data_type}</span>
                </td>
                <td>
                  {column.desc ? (
                    <span className="text-muted">{column.desc}</span>
                  ) : (
                    <span className="text-muted fst-italic">
                      No description
                    </span>
                  )}
                </td>
                <td>
                  {column.foreign_key && (
                    <div className="small">
                      <span className="text-info">FK:</span>{" "}
                      {column.foreign_key.foreign_table}.
                      {column.foreign_key.foreign_column}
                    </div>
                  )}
                  {column.enum_values && column.enum_values.length > 0 && (
                    <div className="small">
                      <span className="text-info">Enum:</span>{" "}
                      {column.enum_values.length} values
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
  return (
    <React.Fragment>
      <li
        className="list-group-item"
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
            className="btn btn-sm btn-outline-secondary me-2"
            onClick={(e) => {
              e.stopPropagation();
              onToggleExpand(table.table_name);
            }}
            aria-label={isExpanded ? "Collapse" : "Expand"}
            tabIndex={-1}
          >
            {isExpanded ? "-" : "+"}
          </button>
          <span style={{ minWidth: 200, flex: 1, fontWeight: 500 }}>
            {table.name || table.table_name}
          </span>
          {table.desc && (
            <span className="text-muted ms-2" style={{ flex: 2 }}>
              {table.desc}
            </span>
          )}
          <span className="text-muted ms-2" style={{ whiteSpace: "nowrap" }}>
            [{table.columns.length} columns]
          </span>
        </div>
      </li>
      {isExpanded && (
        <div>
          <ColumnTable columns={table.columns} />
        </div>
      )}
    </React.Fragment>
  );
};

// Selected data component
interface SelectedDataProps {
  selectedTables: Table[];
  onProceed: () => void;
}

const SelectedData: React.FC<SelectedDataProps> = ({
  selectedTables,
  onProceed,
}) => {
  return (
    <div className="card">
      <div className="card-body">
        <h5 className="card-title">Selected Tables</h5>
        {selectedTables.length === 0 ? (
          <div className="text-muted fst-italic mt-3">No tables selected.</div>
        ) : (
          <ul className="list-group mb-3">
            {selectedTables.map((table) => (
              <li key={table.table_name} className="list-group-item">
                {table.name || table.table_name}
              </li>
            ))}
          </ul>
        )}
        <button
          className="btn btn-primary w-100"
          disabled={selectedTables.length === 0}
          onClick={onProceed}
        >
          Proceed
        </button>
      </div>
    </div>
  );
};

const GROUP_LABEL = "国土数値情報";

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
    alert("Proceed to ChatMapPage with tables: " + tableNames.join(", "));
  };

  return (
    <div className="container-fluid py-4">
      <div className="row">
        <div className="col-9">
          <div className="card">
            <div className="card-body">
              <h5 className="card-title">Database</h5>
              <div className="mb-3">
                <input
                  type="text"
                  className="form-control"
                  placeholder="Search tables or columns..."
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                />
              </div>
              {isLoading && <div className="text-muted">Loading...</div>}
              {error && <div className="text-danger">Failed to load data.</div>}
              <ul className="list-group list-group-flush">
                {!isLoading && !error && filteredTables.length === 0 && (
                  <li className="list-group-item text-muted">
                    No results found.
                  </li>
                )}
                <li key={GROUP_LABEL} className="list-group-item">
                  <div className="d-flex align-items-center mb-1">
                    <button
                      className="btn btn-sm btn-outline-secondary me-2"
                      onClick={() => toggleExpand(GROUP_LABEL)}
                      aria-label={expanded[GROUP_LABEL] ? "Collapse" : "Expand"}
                    >
                      {expanded[GROUP_LABEL] ? "-" : "+"}
                    </button>
                    <span className="fw-bold">{GROUP_LABEL}</span>
                    <span className="text-muted ms-2">
                      [{filteredTables.length} tables]
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
          {selectedTables.length > 0 && (
            <div className="sticky-top" style={{ top: "1.5rem" }}>
              <SelectedData
                selectedTables={selectedTables}
                onProceed={handleProceed}
              />
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default DataNavigatorPage;
