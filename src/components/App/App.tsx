import { AppContext, Categories } from "@/context/AppContext";
import { JSX, useContext } from "react";
import { NavBar } from "../NavBar/NavBar";
import "@/styles/app.css";

export function App(): JSX.Element {
  const { tab, setTab } = useContext(AppContext);

  const navBarItems = [
    {
      label: "Features",
      onSelected: () => {
        setTab(Categories.FEATURE);
      },
      selected: tab == Categories.FEATURE,
    },
    {
      label: "Releases",
      onSelected: () => {
        setTab(Categories.RELEASE);
      },
      selected: tab == Categories.RELEASE,
    },
    {
      label: "Bugfixes",
      onSelected: () => {
        setTab(Categories.BUGFIX);
      },
      selected: tab == Categories.BUGFIX,
    },
  ];

  return (
    <>
      <div id="viewport">
        <NavBar items={navBarItems} />
        <div id="list-layer">
          <table>
            <tbody></tbody>
          </table>
        </div>
      </div>
    </>
  );
}

export default App;
