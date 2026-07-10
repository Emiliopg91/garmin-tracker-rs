import { AppContext } from "@/context/AppContext";
import { useContext } from "react";

export function Loading() {
  const { translate } = useContext(AppContext);

  return (
    <div
      style={{
        position: "absolute",
        top: 0,
        left: 0,
        width: "100%",
        height: "100%",
        zIndex: 10000,
        backgroundColor: "#000000B0",
        display: "flex",
        justifyContent: "center",
        alignItems: "center",
      }}
    >
      <h1>{translate("loading")}</h1>
    </div>
  );
}
