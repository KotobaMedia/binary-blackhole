import React, { useEffect, useMemo, useState } from "react";
import { useAtom, useAtomValue, useSetAtom } from "jotai";
import c from "classnames";
import {
  detailPaneFullscreenAtom,
  detailPaneVisibleAtom,
  layersAtom,
  // selectedFeaturesAtom,
  SQLLayer,
} from "./atoms";
import NavDropdown from "react-bootstrap/NavDropdown";
import BTable from "react-bootstrap/Table";
import {
  createColumnHelper,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import {
  ArrowsCollapseVertical,
  ArrowsExpandVertical,
  X,
} from "react-bootstrap-icons";
import { useQuery } from "../../tools/query";
import "./table.scss";

const LayerTableView: React.FC<{
  layer: SQLLayer;
}> = ({ layer }) => {
  // const selectedFeatures = useAtomValue(selectedFeaturesAtom).filter(
  //   (feature) => feature.layerName === layer.name,
  // );
  // console.log("selectedFeatures", selectedFeatures);
  const { data: resp } = useQuery(layer.sql);
  const [data, columns] = useMemo(() => {
    if (!resp || resp.data.features.length === 0) {
      return [[], []];
    }
    const features = resp.data.features;
    const columnHelper = createColumnHelper<GeoJSON.Feature>();
    const columns = Object.keys(features[0].properties!)
      .filter((key) => {
        if (key.startsWith("_")) {
          return false;
        }
        return true;
      })
      .map((key) =>
        columnHelper.accessor((row) => (row.properties || {})[key], {
          id: key,
        }),
      );
    return [features, columns];
  }, [resp]);
  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    columnResizeMode: "onChange",
    getRowId: (row, idx) => (row.id ?? idx).toString(),
    enableRowSelection: true,
    enableMultiRowSelection: false,
  });
  // useEffect(() => {
  //   table.setRowSelection(
  //     Object.fromEntries(
  //       selectedFeatures
  //         .map((feature) => feature?.feature?.id?.toString())
  //         .filter(Boolean)
  //         .map((id) => [id, true] as const),
  //     ),
  //   );
  // }, [table, selectedFeatures]);

  return (
    <BTable striped bordered hover responsive size="sm" className="data-table">
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
          <tr key={row.id} className={c({ selected: row.getIsSelected() })}>
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
  );
};

const FeatureDetailsPanel: React.FC = () => {
  const setVisible = useSetAtom(detailPaneVisibleAtom);
  const [fullscreen, setFullscreen] = useAtom(detailPaneFullscreenAtom);

  const [selectedLayer, setSelectedLayer] = useState<SQLLayer | undefined>(
    undefined,
  );
  const layers = useAtomValue(layersAtom).filter((layer) => layer.enabled);

  useEffect(() => {
    setSelectedLayer((x) => {
      if (x) {
        return x;
      }
      return layers.length > 0 ? layers[0] : undefined;
    });
  }, [layers]);

  return (
    <div className="feature-details-panel h-100 overflow-auto px-3">
      <nav className="navbar position-sticky top-0 bg-body bg-opacity-75">
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
