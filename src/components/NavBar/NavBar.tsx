import { JSX } from "react";
import { Box } from "@mui/material";

export type NavBarItem = {
  label: JSX.Element;
  selected: boolean;
  onSelected: () => void;
};

export function NavBar({
  leftItems,
  rightItems,
}: {
  leftItems: NavBarItem[];
  rightItems: NavBarItem[];
}): JSX.Element {
  const renderItem = (item: NavBarItem, idx: number) => (
    <Box
      key={"navbar-" + idx}
      onClick={item.onSelected}
      sx={{
        padding: "10px",
        cursor: "pointer",
        bgcolor: item.selected ? "primary.main" : "transparent",
      }}
    >
      {item.label}
    </Box>
  );

  return (
    <Box
      sx={{
        display: "flex",
        flexDirection: "row",
        borderBottom: 1,
        borderColor: "primary.main",
      }}
    >
      {leftItems.map(renderItem)}
      {leftItems.length > 0 && rightItems.length > 0 && (
        <Box key="separator" sx={{ flex: 1 }} />
      )}
      {rightItems.map(renderItem)}
    </Box>
  );
}
