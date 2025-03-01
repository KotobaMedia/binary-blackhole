import { atom } from "jotai";

export type SQLLayer = {
  name: string;
  sql: string;
}

export const layersAtom = atom<SQLLayer[]>([]);
