import { AppContext } from "@/context/AppContext";
import { Backdrop, CircularProgress } from "@mui/material";
import { useContext } from "react";

export function Loading() {
  const { translate } = useContext(AppContext);

  return (
    <Backdrop
      open={true}
      sx={(theme) => ({
        backgroundColor: "#000000D0",
        zIndex: theme.zIndex.drawer + 1,
      })}
    >
      <CircularProgress aria-label={translate("loading")} size="5rem" />
    </Backdrop>
  );
}
