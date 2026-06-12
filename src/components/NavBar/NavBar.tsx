import { JSX } from "react";
import "@/styles/navbar.css";

type NavBarItem = {
  label: string;
  selected: boolean;
  onSelected: () => void;
};

export function NavBar({ items }: { items: NavBarItem[] }): JSX.Element {
  return (
    <div id="navbar">
      {items.map((item) => (
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
