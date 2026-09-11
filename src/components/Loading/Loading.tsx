import { AppContext } from "@/context/AppContext";
import { Backdrop, CircularProgress } from "@mui/material";
import { useContext } from "react";
import "@/styles/Loading/Loading.css";

export function Loading() {
  const { translate } = useContext(AppContext);

  return (
    <Backdrop open={true} className="loading-backdrop">
      <CircularProgress aria-label={translate("loading")} size="5rem" />
    </Backdrop>
  );
}
