import { Workout, WorkoutDetails } from "@/utils/backend/models";
import {
  Button,
  Dialog,
  DialogContent,
  DialogTitle,
  IconButton,
  TextField,
} from "@mui/material";
import CloseIcon from "@mui/icons-material/Close";
import { useContext, useEffect, useState } from "react";
import { WorkoutStepData, WorkoutUtils } from "@/utils/WorkoutUtils";
import { WorkoutSetGroup } from "./helpers/WorkoutSetGroup";
import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { BackendClient } from "@/utils/backend/client";
import { LoadingContext } from "@/context/LoadingContext";

type Props = {
  workout: WorkoutDetails;
  isEdit: boolean;
  onClose: () => void;
};

export function WorkoutSetsModal({ workout, isEdit, onClose }: Props) {
  const { startLoading, finishLoading } = useContext(LoadingContext);
  const { translate, settings } = useContext(I18nSettingsContext);

  useEffect(() => {
    setName(workout.name);
  }, []);

  const [name, setName] = useState(workout?.name);
  const [steps, setSteps] = useState(
    WorkoutUtils.parseWorkoutSteps(workout!.steps, settings.weight_unit),
  );
  const valid = name.length > 0 && WorkoutUtils.validateSteps(steps);

  const swapPosition = (pos1: number, pos2: number) => {
    const tmp = steps[pos1];
    const nSteps = [...steps];
    nSteps[pos1] = nSteps[pos2];
    nSteps[pos2] = tmp;
    setSteps(nSteps);
  };

  const updateStep = (pos: number, nStep: WorkoutStepData) => {
    setSteps(steps.map((step, idx) => (idx == pos ? nStep : step)));
  };

  const addGroup = () => {
    setSteps([...steps, structuredClone(WorkoutUtils.DEFAULT_GROUP)]);
  };

  const deleteGroup = (i: number) => {
    const nSteps = steps.filter((_, idx) => idx != i);
    setSteps(nSteps);
  };

  const saveWorkout = () => {
    if (valid) {
      const newWorkout: Workout = {
        enabled: workout.enabled,
        name,
        steps: WorkoutUtils.toWorkoutSteps(steps, name, settings.weight_unit),
      };
      startLoading();
      BackendClient.saveWorkout(newWorkout)
        .then(() => {
          onClose();
        })
        .finally(() => {
          finishLoading();
        });
    }
  };

  const onNameChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const name = e.target.value;
    setName(name);
  };

  return (
    <Dialog open={true} onClose={onClose} fullWidth maxWidth="md">
      <DialogTitle>
        {isEdit && <span>{name}</span>}
        {!isEdit && (
          <TextField
            placeholder={translate("workout_name")}
            onChange={onNameChange}
            value={name}
          ></TextField>
        )}
        <IconButton onClick={onClose} className="modal-close-button">
          <CloseIcon />
        </IconButton>
      </DialogTitle>

      <DialogContent dividers>
        {steps.map((step, idx) =>
          WorkoutUtils.isStepGroup(step) ? (
            <WorkoutSetGroup
              group={step}
              index={idx}
              sameLevel={steps.length}
              key={`group-${idx}`}
              swapPosition={swapPosition}
              onChange={(g) => updateStep(idx, g)}
              onDelete={() => {
                deleteGroup(idx);
              }}
            />
          ) : (
            <>
              1x
              <br />
              {step.kind}
              <hr />
            </>
          ),
        )}
        <div
          style={{
            display: "flex",
            flexDirection: "row",
            justifyContent: "flex-end",
            alignItems: "center",
            gap: "8px",
          }}
        >
          <Button variant="contained" onClick={addGroup}>
            {translate("add_set")}
          </Button>
          <Button
            color="success"
            variant="contained"
            onClick={saveWorkout}
            disabled={!valid}
          >
            {translate("save_workout")}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
