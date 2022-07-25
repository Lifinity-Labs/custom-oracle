import dayjs, {Dayjs} from "dayjs";
import utc from 'dayjs/plugin/utc';

dayjs.extend(utc);

export const getNow = () => toUTCDayjs();

export const toUTCDayjs = (param?: string | number) => dayjs.utc(param);

export const truncateToHour = (date: Dayjs): Dayjs => {
    return date.startOf('hour');
};
