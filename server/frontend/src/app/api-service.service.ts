import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
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
}
