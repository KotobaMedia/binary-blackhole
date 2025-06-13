import React, { useCallback, useEffect, useMemo, useState } from "react";
import { useAtom, useAtomValue, useSetAtom } from "jotai";
import c from "classnames";
import {
  detailPaneFullscreenAtom,
  detailPaneVisibleAtom,
  enabledLayersAtom,
  selectedFeaturesAtom,
  SQLLayer,
} from "./atoms";
import NavDropdown from "react-bootstrap/NavDropdown";
import BTable from "react-bootstrap/Table";
import {
  ColumnDef,
  createColumnHelper,
  flexRender,
  getCoreRowModel,
  OnChangeFn,
  RowSelectionState,
  useReactTable,
} from "@tanstack/react-table";
import {
  ArrowsCollapseVertical,
  ArrowsExpandVertical,
  X,
} from "react-bootstrap-icons";
import "./table.scss";
import { Form } from "react-bootstrap";
import { useQueryResults } from "../../tools/query";

const LayerTableView: React.FC<{
  layer: SQLLayer;
}> = ({ layer }) => {
  const allSelectedFeatures = useAtomValue(selectedFeaturesAtom);
  const selectedFeatures = useMemo(
    () =>
      allSelectedFeatures.filter((feature) => feature.layer.id === layer.id),
    [allSelectedFeatures, layer.id],
  );
  // const setSelectedFeatures = useSetAtom(selectedFeaturesAtom);
  const rowSelection: RowSelectionState = useMemo(() => {
    const selected = selectedFeatures.reduce((acc, feature) => {
      const rowId = feature.feature.id?.toString();
      if (!rowId) {
        return acc;
      }
      acc[rowId] = true;
      return acc;
    }, {} as RowSelectionState);
    return selected;
  }, [selectedFeatures]);
  const setRowSelection = useCallback<OnChangeFn<RowSelectionState>>(
    (_updater) => {
      // TODO: Implement this
    },
    [],
  );
  const { data: resp } = useQueryResults(layer.id);
  const [data, columns] = useMemo(() => {
    if (!resp || resp.data.length === 0) {
      return [[], []];
    }
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const columnHelper = createColumnHelper<Record<string, any>>();
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    let columns: ColumnDef<Record<string, any>>[] = [
      {
        id: "select-col",
        size: 30,
        header: ({ table }) => (
          <Form.Check
            checked={table.getIsAllRowsSelected()}
            onChange={table.getToggleAllRowsSelectedHandler()} //or getToggleAllPageRowsSelectedHandler
          />
        ),
        cell: ({ row }) => (
          <Form.Check
            checked={row.getIsSelected()}
            disabled={!row.getCanSelect()}
            onChange={row.getToggleSelectedHandler()}
          />
        ),
      },
    ];
    columns = columns.concat(
      Object.keys(resp.data[0])
        .filter((key) => !key.startsWith("_"))
        .map((key) =>
          columnHelper.accessor((row) => row[key], {
            id: key,
          }),
        ),
    );
    return [resp.data, columns];
  }, [resp]);

  useEffect(() => {
    // Scroll the selected row in to view if it is not visible
    const selectedRow = document.querySelector(".table-responsive tr.selected");
    if (selectedRow) {
      const tableBody = selectedRow.closest(".table-responsive");
      if (tableBody) {
        selectedRow.scrollIntoView({
          behavior: "smooth",
          block: "center",
          inline: "nearest",
        });
      }
    }
  }, [rowSelection, selectedFeatures]);

  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    columnResizeMode: "onChange",
    getRowId: (feature, idx) => {
      return (feature._id ?? idx).toString();
    },
    onRowSelectionChange: setRowSelection,
    state: {
      rowSelection,
    },
    enableRowSelection: true,
  });

  return (
    <div className="table-responsive">
      <BTable striped bordered hover size="sm" className="data-table">
        <thead className="position-sticky top-0">
          {table.getHeaderGroups().map((headerGroup) => (
            <tr key={headerGroup.id}>
              {headerGroup.headers.map((header) => (
                <th
                  key={header.id}
                  colSpan={header.colSpan}
                  style={{ width: header.getSize(), position: "relative" }}
                >
                  {header.isPlaceholder
                    ? null
                    : flexRender(
                        header.column.columnDef.header,
                        header.getContext(),
                      )}
                  <div
                    onDoubleClick={() => header.column.resetSize()}
                    onMouseDown={header.getResizeHandler()}
                    onTouchStart={header.getResizeHandler()}
                    className={c(`resizer ltr`, {
                      isResizing: header.column.getIsResizing(),
                    })}
                  />
                </th>
              ))}
            </tr>
          ))}
        </thead>
        <tbody>
          {table.getRowModel().rows.map((row) => (
            <tr
              key={row.id}
              data-row-id={row.id}
              className={c({
                "table-secondary": row.getIsSelected(),
                selected: row.getIsSelected(),
              })}
              onClick={row.getToggleSelectedHandler()}
            >
              {row.getVisibleCells().map((cell) => (
                <td key={cell.id}>
                  {flexRender(cell.column.columnDef.cell, cell.getContext())}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
        <tfoot>
          {table.getFooterGroups().map((footerGroup) => (
            <tr key={footerGroup.id}>
              {footerGroup.headers.map((header) => (
                <th key={header.id} colSpan={header.colSpan}>
                  {header.isPlaceholder
                    ? null
                    : flexRender(
                        header.column.columnDef.footer,
                        header.getContext(),
                      )}
                </th>
              ))}
            </tr>
          ))}
        </tfoot>
      </BTable>
    </div>
  );
};

const FeatureDetailsPanel: React.FC = () => {
  const setVisible = useSetAtom(detailPaneVisibleAtom);
  const [fullscreen, setFullscreen] = useAtom(detailPaneFullscreenAtom);
  const [selectedLayer, setSelectedLayer] = useState<SQLLayer | undefined>(
    undefined,
  );
  const layers = useAtomValue(enabledLayersAtom);
  const selectedFeatures = useAtomValue(selectedFeaturesAtom);

  useEffect(() => {
    setSelectedLayer((x) => {
      if (x) {
        return x;
      }
      return layers.length > 0 ? layers[0] : undefined;
    });
  }, [layers]);

  useEffect(() => {
    // Select the layer of the first selected feature
    if (selectedFeatures.length > 0) {
      const layer = layers.find(
        (layer) => layer.id === selectedFeatures[0].layer.id,
      );
      if (layer) {
        setSelectedLayer(layer);
      }
    }
  }, [layers, selectedFeatures, setSelectedLayer]);

  return (
    <div className="feature-details-panel h-100 d-flex flex-column px-3">
      <nav className="navbar">
        <div className="container-fluid">
          <button className="btn" onClick={() => setFullscreen((x) => !x)}>
            {fullscreen ? <ArrowsCollapseVertical /> : <ArrowsExpandVertical />}
          </button>
          <div>
            <NavDropdown title={selectedLayer?.name} id="layer-dropdown">
              {layers.map((layer) => (
                <NavDropdown.Item
                  key={layer.name}
                  onClick={() => setSelectedLayer(layer)}
                  active={layer.name === selectedLayer?.name}
                >
                  {layer.name}
                </NavDropdown.Item>
              ))}
            </NavDropdown>
          </div>
          <button
            className="btn"
            onClick={() => {
              setVisible(false);
              setFullscreen(false);
            }}
          >
            <X />
          </button>
        </div>
      </nav>

      {selectedLayer && <LayerTableView layer={selectedLayer} />}
    </div>
  );
};

export default FeatureDetailsPanel;
