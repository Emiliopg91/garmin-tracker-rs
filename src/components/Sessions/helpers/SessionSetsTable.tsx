import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { SessionSet } from "@/utils/backend/models";
import { useContext } from "react";
import { TextField } from "@mui/material";
import EmojiEventsIcon from "@mui/icons-material/EmojiEvents";

type Props = {
  exercises: string[];
  groupedSeries: Record<string, SessionSet[]>;
  onUpdateSerie: (
    exercise: string,
    idx: number,
    field: "reps" | "weight",
    value: number,
  ) => void;
};

export function SessionSetsTable({
  exercises,
  groupedSeries,
  onUpdateSerie,
}: Props) {
  const { translate, getWeightUnit } = useContext(I18nSettingsContext);

  const updateSerieReps = (exercise: string, idx: number, newVal: string) => {
    let reps = parseInt(newVal);
    if (isNaN(reps)) {
      reps = 0;
    }
    onUpdateSerie(exercise, idx, "reps", reps);
  };

  const updateSerieWeight = (exercise: string, idx: number, newVal: string) => {
    let weight = parseFloat(newVal);
    if (isNaN(weight)) {
      weight = 0;
    }
    onUpdateSerie(exercise, idx, "weight", weight);
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
            groupedSeries[exercise].map((serie, idx) => (
              <tr key={`${exercise}-${idx}`}>
                {idx === 0 && (
                  <td
                    className="divider-bottom"
                    rowSpan={groupedSeries[exercise].length}
                  >
                    {translate("exercise_" + serie.ex_cat + "_" + serie.ex_id)}
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
            )),
          )}
        </tbody>
      </table>
    </div>
  );
}
