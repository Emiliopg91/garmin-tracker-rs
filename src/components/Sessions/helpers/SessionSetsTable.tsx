import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { SessionSet } from "@/utils/backend/models";
import { useContext } from "react";
import { Autocomplete, TextField } from "@mui/material";
import EmojiEventsIcon from "@mui/icons-material/EmojiEvents";
import { ExerciseOption, useExerciseCatalog } from "@/hooks/useExerciseCatalog";

type Props = {
  series: SessionSet[];
  onUpdateSerieRepsWeight: (
    idx: number,
    field: "reps" | "weight",
    value: number,
  ) => void;
  onUpdateSerieExercise: (idx: number, category: number, id: number) => void;
};

export function SessionSetsTable({
  series,
  onUpdateSerieRepsWeight,
  onUpdateSerieExercise,
}: Props) {
  const { translate, getWeightUnit, formatDuration } =
    useContext(I18nSettingsContext);
  // On failure the exercise name is rendered as plain text instead of a selector
  const exercisesCatalog = useExerciseCatalog(() => {
    console.warn("Could not load exercises catalog");
  });

  const updateSerieReps = (idx: number, newVal: string) => {
    let reps = parseInt(newVal);
    if (isNaN(reps)) {
      reps = 0;
    }
    onUpdateSerieRepsWeight(idx, "reps", reps);
  };

  const updateSerieWeight = (idx: number, newVal: string) => {
    let weight = parseFloat(newVal);
    if (isNaN(weight)) {
      weight = 0;
    }
    onUpdateSerieRepsWeight(idx, "weight", weight);
  };

  const onExerciseChange = (idx: number, exerciseOption: ExerciseOption) => {
    onUpdateSerieExercise(idx, exerciseOption.ex_cat, exerciseOption.ex_id);
  };

  let serieIdx = 1;
  let firstOfSerie = true;

  return (
    <div className="session-sets-container">
      <table>
        <colgroup>
          <col className="col-320" />
          <col className="col-230" />
        </colgroup>
        {series
          .sort((a, b) => a.idx - b.idx)
          .map((serie, idx) => {
            const selected = exercisesCatalog.find(
              (e) => e.ex_cat == serie.ex_cat && e.ex_id == serie.ex_id,
            );
            const isFirst = firstOfSerie;
            const thisSerie = serieIdx;
            firstOfSerie = serie.rest;
            if (serie.rest) {
              serieIdx++;
            }

            return (
              <>
                {isFirst && (
                  <thead>
                    <tr className="divider-bottom">
                      <th colSpan={2}>
                        {translate("set_x", [thisSerie.toString()])}
                      </th>
                    </tr>
                  </thead>
                )}

                <tbody>
                  <tr>
                    {serie.rest && (
                      <td colSpan={2}>
                        <div
                          style={{ textAlign: "center", paddingTop: "10px" }}
                        >
                          <span>
                            <small>
                              {"" + translate("step_type_rest") + ": "}
                              {formatDuration(serie.duration)}
                            </small>
                          </span>
                        </div>
                        <div className="divider-bottom session-sets-rest-divider" />
                      </td>
                    )}

                    {!serie.rest && (
                      <>
                        <td>
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
                              sx={{ width: 300, mt: 1 }}
                              options={exercisesCatalog}
                              value={selected}
                              disableClearable
                              groupBy={(opt) =>
                                translate("exercise_" + opt.ex_cat)
                              }
                              getOptionKey={(opt) =>
                                `${opt.ex_cat}-${opt.ex_id}`
                              }
                              isOptionEqualToValue={(a, b) =>
                                a.ex_cat == b.ex_cat && a.ex_id == b.ex_id
                              }
                              onChange={(_, opt) => onExerciseChange(idx, opt)}
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
                        <td>
                          <TextField
                            label={translate("repetitions")}
                            type="number"
                            size="small"
                            value={serie.reps}
                            sx={{ width: 100, mt: 1 }}
                            style={{ marginTop: "5px" }}
                            slotProps={{
                              htmlInput: {
                                className: "no-spinner",
                                min: 0,
                              },
                            }}
                            onChange={(e) => {
                              updateSerieReps(idx, e.target.value);
                            }}
                          />
                          <TextField
                            label={
                              translate("weight") + " (" + getWeightUnit() + ")"
                            }
                            type="number"
                            size="small"
                            sx={{ width: 100, px: 1, mt: 1 }}
                            style={{ marginTop: "5px" }}
                            value={serie.weight?.toString()}
                            slotProps={{
                              htmlInput: {
                                className: "no-spinner",
                                min: 0,
                              },
                            }}
                            onChange={(e) => {
                              updateSerieWeight(idx, e.target.value);
                            }}
                          />
                          {serie.pr && (
                            <EmojiEventsIcon
                              fontSize="small"
                              className="trophy-icon"
                              style={{ marginTop: "15px" }}
                            />
                          )}
                        </td>
                      </>
                    )}
                  </tr>
                </tbody>
              </>
            );
          })}
      </table>
    </div>
  );
}
