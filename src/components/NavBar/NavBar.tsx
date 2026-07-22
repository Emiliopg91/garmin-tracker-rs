import { JSX } from "react";
import "@/styles/navbar.css";

type NavBarItem = {
  label: string;
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
  return (
    <div id="navbar">
      {leftItems.map((item) => (
        <div
          key={item.label}
          className={`navbar-item ${item.selected ? "selected" : ""}`}
          onClick={item.onSelected}
        >
          {item.label}
        </div>
      ))}
      {leftItems.length > 0 && rightItems.length > 0 && (
        <div key={`separator`} className="navbar-separator" />
      )}
      {rightItems.map((item) => (
        <div
          key={item.label}
          className={`navbar-item ${item.selected ? "selected" : ""}`}
          onClick={item.onSelected}
        >
          {item.label}
        </div>
      ))}
    </div>
  );
}
