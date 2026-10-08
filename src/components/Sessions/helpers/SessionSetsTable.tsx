import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { SessionSet } from "@/utils/backend/models";
import { useContext } from "react";
import { Autocomplete, TextField } from "@mui/material";
import EmojiEventsIcon from "@mui/icons-material/EmojiEvents";
import { ExerciseOption, useExerciseCatalog } from "@/hooks/useExerciseCatalog";

type Props = {
  exercises: string[];
  groupedSeries: Record<string, SessionSet[]>;
  onUpdateSerieRepsWeight: (
    exercise: string,
    idx: number,
    field: "reps" | "weight",
    value: number,
  ) => void;
  onUpdateSerieExercise: (
    exercise: string,
    category: number,
    id: number,
  ) => void;
};

export function SessionSetsTable({
  exercises,
  groupedSeries,
  onUpdateSerieRepsWeight,
  onUpdateSerieExercise,
}: Props) {
  const { translate, getWeightUnit } = useContext(I18nSettingsContext);
  // On failure the exercise name is rendered as plain text instead of a selector
  const exercisesCatalog = useExerciseCatalog(() => {
    console.warn("Could not load exercises catalog");
  });

  const updateSerieReps = (exercise: string, idx: number, newVal: string) => {
    let reps = parseInt(newVal);
    if (isNaN(reps)) {
      reps = 0;
    }
    onUpdateSerieRepsWeight(exercise, idx, "reps", reps);
  };

  const updateSerieWeight = (exercise: string, idx: number, newVal: string) => {
    let weight = parseFloat(newVal);
    if (isNaN(weight)) {
      weight = 0;
    }
    onUpdateSerieRepsWeight(exercise, idx, "weight", weight);
  };

  const onExerciseChange = (
    exercise: string,
    exerciseOption: ExerciseOption,
  ) => {
    onUpdateSerieExercise(
      exercise,
      exerciseOption.ex_cat,
      exerciseOption.ex_id,
    );
  };

  return (
    <div className="session-sets-container">
      <table>
        <colgroup>
          <col className="col-320" />
          <col className="col-230" />
        </colgroup>

        <thead>
          <tr className="divider-bottom">
            <th>{translate("exercise")}:</th>
            <th>{translate("series")}:</th>
          </tr>
        </thead>
        <tbody>
          {exercises.map((exercise) =>
            groupedSeries[exercise].map((serie, idx) => {
              const selected =
                idx === 0
                  ? exercisesCatalog.find(
                      (e) => e.ex_cat == serie.ex_cat && e.ex_id == serie.ex_id,
                    )
                  : undefined;
              return (
                <tr key={`${exercise}-${idx}`}>
                  {idx === 0 && (
                    <td
                      className="divider-bottom"
                      rowSpan={groupedSeries[exercise].length}
                    >
                      {!selected && (
                        <>
                          {translate(
                            "exercise_" + serie.ex_cat + "_" + serie.ex_id,
                          )}
                        </>
                      )}
                      {selected && (
                        <Autocomplete
                          size="small"
                          sx={{ width: 300 }}
                          options={exercisesCatalog}
                          value={selected}
                          disableClearable
                          groupBy={(opt) => translate("exercise_" + opt.ex_cat)}
                          getOptionKey={(opt) => `${opt.ex_cat}-${opt.ex_id}`}
                          isOptionEqualToValue={(a, b) =>
                            a.ex_cat == b.ex_cat && a.ex_id == b.ex_id
                          }
                          onChange={(_, opt) => onExerciseChange(exercise, opt)}
                          renderInput={(params) => (
                            <TextField
                              {...params}
                              multiline
                              label={translate("select_exercise")}
                            />
                          )}
                        />
                      )}
                    </td>
                  )}

                  <td
                    className={[
                      idx === groupedSeries[exercise].length - 1
                        ? "divider-bottom group-cell-last"
                        : undefined,
                      idx === 0 ? "group-cell-first" : undefined,
                    ]
                      .filter(Boolean)
                      .join(" ")}
                  >
                    <TextField
                      label={translate("repetitions")}
                      type="number"
                      size="small"
                      value={serie.reps}
                      sx={{ width: 100 }}
                      style={{ marginTop: "5px" }}
                      slotProps={{
                        htmlInput: {
                          className: "no-spinner",
                          min: 0,
                        },
                      }}
                      onChange={(e) => {
                        updateSerieReps(exercise, idx, e.target.value);
                      }}
                    />
                    <TextField
                      label={translate("weight") + " (" + getWeightUnit() + ")"}
                      type="number"
                      size="small"
                      sx={{ width: 100, px: 1 }}
                      style={{ marginTop: "5px" }}
                      value={serie.weight?.toString()}
                      slotProps={{
                        htmlInput: {
                          className: "no-spinner",
                          min: 0,
                        },
                      }}
                      onChange={(e) => {
                        updateSerieWeight(exercise, idx, e.target.value);
                      }}
                    />
                    {serie.pr && (
                      <EmojiEventsIcon
                        className="trophy-icon"
                        style={{ marginTop: "15px" }}
                      />
                    )}
                  </td>
                </tr>
              );
            }),
          )}
        </tbody>
      </table>
    </div>
  );
}
