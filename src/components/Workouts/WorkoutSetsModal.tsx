import { StepType, WorkoutDetails } from "@/utils/backend/models";
import {
  Button,
  Dialog,
  DialogContent,
  DialogTitle,
  IconButton,
} from "@mui/material";
import CloseIcon from "@mui/icons-material/Close";
import { useContext, useEffect, useState } from "react";
import {
  LeafStep,
  StepGroup,
  WorkoutStepData,
  WorkoutUtils,
} from "@/utils/WorkoutUtils";
import { WorkoutSetGroup } from "./helpers/WorkoutSetGroup";
import { I18nSettingsContext } from "@/context/I18nSettingsContext";

type Props = {
  workout: WorkoutDetails;
  onClose: () => void;
};

export function WorkoutSetsModal({ workout, onClose }: Props) {
  //const { startLoading, finishLoading } = useContext(LoadingContext);
  const { translate } = useContext(I18nSettingsContext);

  useEffect(() => {
    setName(workout.name);
  }, []);

  const [name, setName] = useState(workout?.name);
  const [steps, setSteps] = useState(
    WorkoutUtils.parseWorkoutSteps(workout!.steps),
  );

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
    const defaultLeaf: LeafStep = {
      kind: StepType.Rest,
      time: 120,
      ex_cat: null,
      ex_id: null,
      reps: null,
      weight: null,
    };
    const group: StepGroup = {
      laps: 1,
      wrapped: [defaultLeaf],
    };
    setSteps([...steps, group]);
  };

  const deleteGroup = (i: number) => {
    const nSteps = steps.filter((_, idx) => idx != i);
    setSteps(nSteps);
  };

  return (
    <Dialog open={true} onClose={onClose} fullWidth maxWidth="md">
      <DialogTitle>
        <span>{name}</span>
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
          <Button color="success" variant="contained">
            {translate("save_workout")}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
