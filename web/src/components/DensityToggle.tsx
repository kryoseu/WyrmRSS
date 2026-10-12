import { TbLayoutList, TbList } from "react-icons/tb";
import { useTheme } from "../hooks/useTheme";

/** Toolbar shortcut for Settings › Appearance › Post list. */
export function DensityToggle() {
  const { density, setDensity } = useTheme();
  const card = density === "card";

  return (
    <button
      className="posts-refresh-btn"
      aria-pressed={card}
      onClick={() => setDensity(card ? "title" : "card")}
      title={card ? "Cards — click to show titles only" : "Title only — click to show cards"}
    >
      {card ? <TbLayoutList /> : <TbList />}
    </button>
  );
}
