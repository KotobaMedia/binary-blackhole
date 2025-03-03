import { atom } from "jotai";

export type SQLLayer = {
  name: string;
  sql: string;
  enabled: boolean;
}

export const layersAtom = atom<SQLLayer[]>([]);
