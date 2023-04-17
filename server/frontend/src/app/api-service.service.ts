import { Injectable } from '@angular/core';
import { HttpClient, HttpHeaders } from '@angular/common/http';
import { Observable, catchError, of } from 'rxjs';
import { ProfileConfig } from './profile-config';

@Injectable({
  providedIn: 'root'
})
export class ApiServiceService {

  constructor(private http: HttpClient) { }

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

  createBlankProfileConfig(name: string): Observable<ProfileConfig | undefined> {
    const httpOptions = {
      headers: new HttpHeaders({ 'Content-Type': 'application/json' })
    };

    return this.http.post<ProfileConfig | undefined>(`/api/profiles/create/${name}`, null, httpOptions).pipe(
      catchError(err => {
        console.error('Couldn\' create new ProfileConfig. Got:');
        console.error(err);
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
      catchError(err => {
        console.error('Couldn\'t update profileConfig, got:');
        console.error(err);
        return of(profileConfig);
      })
    );
  }
}
