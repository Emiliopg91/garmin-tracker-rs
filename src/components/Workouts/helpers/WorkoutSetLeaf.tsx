import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { StepType } from "@/utils/backend/models";
import { LeafStep } from "@/utils/WorkoutUtils";
import {
  Box,
  FormControl,
  IconButton,
  InputLabel,
  MenuItem,
  Select,
  SelectChangeEvent,
  TextField,
  Tooltip,
  Typography,
} from "@mui/material";
import { useContext, useId } from "react";
import ArrowUpwardIcon from "@mui/icons-material/ArrowUpward";
import ArrowDownwardIcon from "@mui/icons-material/ArrowDownward";
import ClearIcon from "@mui/icons-material/Clear";
import { UnitUtils } from "@/utils/UnitUtils";

type Props = {
  leaf: LeafStep;
  index: number;
  sameLevel: number;
  onDelete: () => void;
  swapPosition: (pos1: number, pos2: number) => void;
  onChange: (leaf: LeafStep) => void;
};

enum LimitType {
  Time,
  Lap,
  Reps,
}

export function WorkoutSetLeaf({
  leaf,
  index,
  sameLevel,
  onDelete,
  swapPosition,
  onChange,
}: Props) {
  const { translate, settings } = useContext(I18nSettingsContext);
  const typeLabelId = useId();
  const limitLabelId = useId();
  const isExercise = leaf.kind == StepType.Exercise;
  const limit = isExercise
    ? leaf.reps != null
      ? LimitType.Reps
      : leaf.time != null
        ? LimitType.Time
        : LimitType.Lap
    : leaf.time != null
      ? LimitType.Time
      : LimitType.Lap;
  const minutes = leaf.time != null ? Math.floor(leaf.time / 60) : 0;
  const seconds = leaf.time != null ? leaf.time % 60 : 0;

  const onTypeChange = (e: SelectChangeEvent<string>) => {
    onChange({ ...leaf, kind: e.target.value as StepType });
  };

  const onLimitChange = (e: SelectChangeEvent<number>) => {
    const obj = { ...leaf };
    switch (e.target.value as LimitType) {
      case LimitType.Lap:
        obj.reps = null;
        obj.time = null;
        break;
      case LimitType.Reps:
        obj.reps = 1;
        obj.time = null;
        break;
      case LimitType.Time:
        obj.time = 60;
        obj.reps = null;
        break;
    }
    onChange(obj);
  };

  const onWeightChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const weight = parseFloat(e.target.value);
    onChange({
      ...leaf,
      weight: isNaN(weight) ? null : Math.round(weight * 10) / 10,
    });
  };

  const onRepsChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const reps = parseInt(e.target.value);
    onChange({
      ...leaf,
      reps: isNaN(reps) ? null : Math.round(reps * 10) / 10,
    });
  };

  const onMinutesChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const minutes = parseInt(e.target.value) || 0;
    onChange({
      ...leaf,
      time: minutes * 60 + seconds,
    });
  };

  const onSecondsChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const seconds = parseInt(e.target.value) || 0;
    onChange({
      ...leaf,
      time: minutes * 60 + seconds,
    });
  };

  return (
    <>
      <Box
        sx={{
          display: "flex",
          flexWrap: "wrap",
          alignItems: "center",
          gap: 1,
          px: 2,
          py: 1,
        }}
      >
        <FormControl size="small" sx={{ width: 120 }}>
          <InputLabel id={typeLabelId}>{translate("step_type")}</InputLabel>
          <Select
            labelId={typeLabelId}
            value={leaf.kind}
            label={translate("step_type")}
            onChange={onTypeChange}
            MenuProps={{
              slotProps: { paper: { style: { maxHeight: 300 } } },
            }}
          >
            <MenuItem value={StepType.Exercise}>
              {translate("step_type_exercise")}
            </MenuItem>
            <MenuItem value={StepType.Rest}>
              {translate("step_type_rest")}
            </MenuItem>
          </Select>
        </FormControl>
        {isExercise && (
          <Typography>
            {translate("exercise_" + leaf.ex_cat + "_" + leaf.ex_id)}
          </Typography>
        )}
      </Box>

      <Box
        sx={{
          display: "flex",
          flexDirection: "row",
          alignItems: "center",
          pb: 1,
        }}
      >
        <Box
          sx={{
            display: "flex",
            flex: 1,
            gap: 0.5,
            flexDirection: "row",
            justifyContent: "flex-start",
            pl: 2,
            marginTop: "5px",
          }}
        >
          <FormControl size="small" sx={{ width: 140, marginRight: "20px" }}>
            <InputLabel id={limitLabelId}>
              {translate("step_limit_type")}
            </InputLabel>
            <Select
              labelId={limitLabelId}
              value={limit}
              label={translate("step_limit_type")}
              onChange={onLimitChange}
              MenuProps={{
                slotProps: { paper: { style: { maxHeight: 300 } } },
              }}
            >
              <MenuItem value={LimitType.Time}>
                {translate("limit_type_time")}
              </MenuItem>
              <MenuItem value={LimitType.Lap}>
                {translate("limit_type_lap")}
              </MenuItem>
              <MenuItem value={LimitType.Reps}>
                {translate("limit_type_reps")}
              </MenuItem>
            </Select>
          </FormControl>

          {isExercise && limit == LimitType.Reps && (
            <TextField
              label={translate("repetitions")}
              type="number"
              size="small"
              sx={{ width: 100 }}
              value={leaf.reps ?? ""}
              slotProps={{
                htmlInput: {
                  className: "no-spinner",
                  inputMode: "decimal",
                  min: 1,
                  max: 999,
                  step: 1,
                },
              }}
              onChange={onRepsChange}
            />
          )}
          {limit == LimitType.Time && (
            <>
              <TextField
                label={translate("minutes")}
                type="number"
                size="small"
                sx={{ width: 100 }}
                value={minutes ?? ""}
                slotProps={{
                  htmlInput: {
                    className: "no-spinner",
                    inputMode: "decimal",
                    min: 0,
                    step: 1,
                  },
                }}
                onChange={onMinutesChange}
              />
              <TextField
                label={translate("seconds")}
                type="number"
                size="small"
                sx={{ width: 100 }}
                value={seconds ?? ""}
                slotProps={{
                  htmlInput: {
                    className: "no-spinner",
                    inputMode: "decimal",
                    min: 0,
                    max: 59,
                    step: 1,
                  },
                }}
                onChange={onSecondsChange}
              />
            </>
          )}

          {isExercise && (
            <TextField
              label={
                translate("weight") +
                " (" +
                UnitUtils.getUnit(settings.weight_unit) +
                ")"
              }
              type="number"
              size="small"
              sx={{ width: 100 }}
              value={leaf.weight ?? ""}
              slotProps={{
                htmlInput: {
                  className: "no-spinner",
                  inputMode: "decimal",
                  min: 0,
                  max: 999,
                  step: 0.1,
                },
              }}
              onChange={onWeightChange}
            />
          )}
        </Box>

        <Box
          sx={{
            display: "flex",
            gap: 0.5,
            flexDirection: "row",
            justifyContent: "flex-end",
          }}
        >
          <Tooltip title={translate("move_up")}>
            <span>
              <IconButton
                size="small"
                color="inherit"
                disabled={index == 0}
                onClick={() => swapPosition(index, index - 1)}
              >
                <ArrowUpwardIcon fontSize="small" />
              </IconButton>
            </span>
          </Tooltip>
          <Tooltip title={translate("move_down")}>
            <span>
              <IconButton
                size="small"
                color="inherit"
                disabled={index == sameLevel - 1}
                onClick={() => swapPosition(index, index + 1)}
              >
                <ArrowDownwardIcon fontSize="small" />
              </IconButton>
            </span>
          </Tooltip>
          <Tooltip title={translate("delete")}>
            <span>
              <IconButton
                size="small"
                color="error"
                disabled={sameLevel <= 1}
                onClick={onDelete}
              >
                <ClearIcon fontSize="small" />
              </IconButton>
            </span>
          </Tooltip>
        </Box>
      </Box>
    </>
  );
}
