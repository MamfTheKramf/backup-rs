import { Injectable } from '@angular/core';
import { HttpClient, HttpHeaders } from '@angular/common/http';
import { Observable, catchError, of, tap } from 'rxjs';
import { ProfileConfig } from './profile-config';
import { MessageService } from './message.service';
import { Message, MessageType } from 'Message';

@Injectable({
  providedIn: 'root'
})
export class ApiServiceService {

  constructor(private http: HttpClient, private readonly messageService: MessageService) { }

  /**
   * Fetches the existing `ProfileConfig`s
   * @returns An array of all `ProfileConfig`s. If there was an error, an empty array is returned.
   */
  getProfileConfigs(): Observable<ProfileConfig[]> {
    return this.http.get<ProfileConfig[]>('/api/profiles').pipe(
      catchError(err => {
        console.error('Got error:');
        console.error(err);
        return of([]);
      })
    );
  }

  deleteProfileConfig(uuid: string): Observable<void> {
    return this.http.delete<void>(`/api/profiles/uuid/${uuid}`).pipe(
      tap(() => this.messageService.sendMsg(new Message(MessageType.Info, 'Profil gelöscht'))),
      catchError(err => {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        this.messageService.sendMsg(new Message(MessageType.Error, `Profil konnte nicht gelöscht werden: ${(err as any).error}`));
        return of();
      })
    );
  }

  createBlankProfileConfig(name: string): Observable<ProfileConfig | undefined> {
    const httpOptions = {
      headers: new HttpHeaders({ 'Content-Type': 'application/json' })
    };

    return this.http.post<ProfileConfig | undefined>(`/api/profiles/create/${name}`, null, httpOptions).pipe(
      tap(() => this.messageService.sendMsg(new Message(MessageType.Info, 'Neues Profil erfolgreich erzeugt'))),
      catchError(err => {
        console.error('Couldn\' create new ProfileConfig. Got:');
        console.error(err);
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        this.messageService.sendMsg(new Message(MessageType.Error, `Profil konnte nicht erzeugt werden: ${(err as any).error}`));
        return of(undefined);
      })
    );
  }

  /**
   * Updates the provided `ProfileConfig`
   * @returns The version of the `ProfileConfig` that is now stored
   */
  updateProfileConfigs(profileConfig: ProfileConfig): Observable<ProfileConfig> {
    const httpOptions = {
      headers: new HttpHeaders({ 'Content-Type': 'application/json' })
    };
    return this.http.put<ProfileConfig>(`/api/profiles/uuid/${profileConfig.uuid}`, profileConfig, httpOptions).pipe(
      tap(() => this.messageService.sendMsg(new Message(MessageType.Info, 'Profil erfolgreich aktualisiert'))),
      catchError(err => {
        console.error('Couldn\'t update profileConfig, got:');
        console.error(err);
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        this.messageService.sendMsg(new Message(MessageType.Error, `Profil konnte nicht aktualisiert werden: ${(err as any).error}`));
        return of(profileConfig);
      })
    );
  }
}
