export type LibraryCollectionSelection = {
  viewCollectionId: string | null;
  addTargetCollectionId: string | null;
};

export type LibraryCollectionSelectionAction =
  | { type: "browse"; collectionId: string | null }
  | { type: "choose-add-target"; collectionId: string | null };

export const initialLibraryCollectionSelection: LibraryCollectionSelection = {
  viewCollectionId: null,
  addTargetCollectionId: null,
};

export function reduceLibraryCollectionSelection(
  state: LibraryCollectionSelection,
  action: LibraryCollectionSelectionAction,
): LibraryCollectionSelection {
  switch (action.type) {
    case "browse":
      return { ...state, viewCollectionId: action.collectionId };
    case "choose-add-target":
      return { ...state, addTargetCollectionId: action.collectionId };
  }
}
