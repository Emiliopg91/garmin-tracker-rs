import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { SessionListItem } from "@/utils/backend/models";
import { TimeUtils } from "@/utils/TimeUtils";
import { useContext } from "react";
import EmojiEventsIcon from "@mui/icons-material/EmojiEvents";

type Props = {
  session: SessionListItem;
  onSelect: (timestamp: number) => void;
};

export function SessionRow({ session, onSelect }: Props) {
  const { translate } = useContext(I18nSettingsContext);

  return (
    <tr onClick={() => onSelect(session.timestamp)} className="clickable-row">
      <td>{session.has_record && <EmojiEventsIcon className="trophy-icon" />}</td>
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
