import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { SessionListItem } from "@/utils/backend/models";
import { TimeUtils } from "@/utils/TimeUtils";
import { useContext } from "react";
import EmojiEventsIcon from "@mui/icons-material/EmojiEvents";
import { Checkbox } from "@mui/material";

type Props = {
  session: SessionListItem;
  onSelect: (timestamp: number) => void;
  compareName: string | undefined;
  onToggleCompare: (
    event: React.MouseEvent<HTMLButtonElement>,
    session: SessionListItem,
  ) => void;
};

export function SessionRow({
  session,
  compareName,
  onSelect,
  onToggleCompare,
}: Props) {
  const { translate } = useContext(I18nSettingsContext);

  return (
    <tr onClick={() => onSelect(session.timestamp)} className="clickable-row">
      <td>
        {session.has_sets && (
          <Checkbox
            onClick={(event) => onToggleCompare(event, session)}
            className="compare-checkbox"
            disabled={compareName !== undefined && compareName !== session.name}
          />
        )}
      </td>
      <td>
        {session.has_record && <EmojiEventsIcon className="trophy-icon" />}
      </td>
      <td>{TimeUtils.formatTimeDate(session.timestamp)}</td>
      <td>
        {translate("sport_" + session.sport) +
          " - " +
          translate("sport_" + session.sport + "_" + session.sub_sport)}
      </td>
      <td>{session.name}</td>
      <td>{session.active_calories}</td>
      <td>{session.training_load}</td>
    </tr>
  );
}
