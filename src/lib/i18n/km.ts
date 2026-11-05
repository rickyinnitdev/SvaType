import app from '../../../content/app.json';

export const km = { appName: app.name };
export type Messages = { [K in keyof typeof km]: string };
