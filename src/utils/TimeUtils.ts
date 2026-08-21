export class TimeUtils {
  public static formatDuration(seconds: number): string {
    if (seconds == 0) {
      return "0:00";
    }

    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = Math.floor(seconds % 60);

    let res: string;
    if (h > 0) {
      res = `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
    } else if (m > 0) {
      res = `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
    } else {
      res = `${s}`;
    }

    while (res.startsWith("0")) {
      res = res.slice(1);
    }

    return res;
  }

  public static formatTimeDate(date: number): string {
    const datetime = new Date(date * 1000);

    const hours = String(datetime.getHours()).padStart(2, "0");
    const minutes = String(datetime.getMinutes()).padStart(2, "0");
    const day = String(datetime.getDate()).padStart(2, "0");
    const month = String(datetime.getMonth() + 1).padStart(2, "0");
    const year = String(datetime.getFullYear()).padStart(4, "0");

    return `${hours}:${minutes} ${day}/${month}/${year}`;
  }

  public static formatDate(date: number): string {
    const datetime = new Date(date * 1000);

    const day = String(datetime.getDate()).padStart(2, "0");
    const month = String(datetime.getMonth() + 1).padStart(2, "0");
    const year = String(datetime.getFullYear()).padStart(4, "0");

    return `${day}/${month}/${year}`;
  }

  public static parseLocalDateTime(dateStr: string): Date {
    const match = dateStr.match(/^(\d{2}):(\d{2}) (\d{2})\/(\d{2})\/(\d{4})$/);

    if (!match) {
      throw new Error("Wrong date format");
    }

    const [, hourStr, minStr, dayStr, monthStr, yearStr] = match;

    const hour = Number(hourStr);
    const minute = Number(minStr);
    const day = Number(dayStr);
    const month = Number(monthStr);
    const year = Number(yearStr);

    const local = new Date(year, month - 1, day, hour, minute, 0);

    if (
      local.getFullYear() !== year ||
      local.getMonth() !== month - 1 ||
      local.getDate() !== day ||
      local.getHours() !== hour ||
      local.getMinutes() !== minute
    ) {
      throw new Error("Wrong date format");
    }

    return local;
  }
}
