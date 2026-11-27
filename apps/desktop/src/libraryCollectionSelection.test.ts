import { describe, expect, it } from "vitest";
import {
  initialLibraryCollectionSelection,
  reduceLibraryCollectionSelection,
} from "./libraryCollectionSelection";

describe("library collection selection", () => {
  it("keeps the browsed collection when choosing an add destination", () => {
    const browsingCrate = reduceLibraryCollectionSelection(
      initialLibraryCollectionSelection,
      { type: "browse", collectionId: "crate-1" },
    );

    expect(
      reduceLibraryCollectionSelection(browsingCrate, {
        type: "choose-add-target",
        collectionId: "playlist-1",
      }),
    ).toEqual({
      viewCollectionId: "crate-1",
      addTargetCollectionId: "playlist-1",
    });
  });

  it("clears the browsed view without losing the add destination", () => {
    const targetSelected = reduceLibraryCollectionSelection(
      initialLibraryCollectionSelection,
      { type: "choose-add-target", collectionId: "playlist-1" },
    );

    expect(
      reduceLibraryCollectionSelection(targetSelected, {
        type: "browse",
        collectionId: null,
      }),
    ).toEqual({
      viewCollectionId: null,
      addTargetCollectionId: "playlist-1",
    });
  });
});
