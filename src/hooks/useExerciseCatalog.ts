import { BackendClient } from "@/utils/backend/client";
import { useContext, useEffect, useMemo, useRef, useState } from "react";
import { useLoadingTask } from "./useLoadingTask";
import { I18nSettingsContext } from "@/context/I18nSettingsContext";

export type ExerciseOption = {
  label: string;
  ex_cat: number;
  ex_id: number;
};

// Fetches the catalog from backend and structures it
export function useExerciseCatalog(onFailure: () => void) {
  const withLoading = useLoadingTask();
  const { translate } = useContext(I18nSettingsContext);
  const [rawCatalog, setRawCatalog] = useState<Record<number, number[]> | null>(
    null,
  );

  const onFailureRef = useRef(onFailure);
  onFailureRef.current = onFailure;

  useEffect(() => {
    withLoading(BackendClient.getExercisesCatalog())
      .then(setRawCatalog)
      .catch(() => onFailureRef.current());
  }, []);

  const exercisesCatalog = useMemo<ExerciseOption[] | null>(() => {
    if (!rawCatalog) return null;
    const catalog: ExerciseOption[] = [];
    const categoryLabels = new Map<number, string>();
    for (const [cat, ex_ids] of Object.entries(rawCatalog)) {
      const ex_cat = Number(cat);
      categoryLabels.set(ex_cat, translate("exercise_" + ex_cat));
      for (const ex_id of ex_ids) {
        catalog.push({
          ex_cat,
          ex_id,
          label: translate("exercise_" + ex_cat + "_" + ex_id),
        });
      }
    }
    // Sorted by category first, as Autocomplete's groupBy expects grouped options to be contiguous
    return catalog.sort(
      (a, b) =>
        categoryLabels
          .get(a.ex_cat)!
          .localeCompare(categoryLabels.get(b.ex_cat)!) ||
        a.label.localeCompare(b.label),
    );
  }, [rawCatalog, translate]);

  return exercisesCatalog;
}
