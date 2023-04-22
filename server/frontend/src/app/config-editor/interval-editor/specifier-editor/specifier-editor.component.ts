import { Component, Input, OnInit } from '@angular/core';
import { SPECIFIER_KINDS, Specifier, SpecifierKind, isDay, isMonth } from 'src/app/profile-config';

@Component({
  selector: 'app-specifier-editor',
  templateUrl: './specifier-editor.component.html',
  styleUrls: ['./specifier-editor.component.scss']
})
export class SpecifierEditorComponent<MIN, MAX> implements OnInit {
  @Input() specifier!: Specifier<MIN, MAX>;
  @Input() offset = 0;

  readonly specifierKinds = SPECIFIER_KINDS;
  selectedKind = '';

  ngOnInit(): void {
    this.initSelectedKind();
  }

  initSelectedKind(): void {
    this.selectedKind = this.toKindString(this.specifier.kind); 
  }

  /**
   * Sets the `kind` of `specifier` to a new object of the `selectedKind`
   */
  changeKind(): void {
    // do nothing of the kind doesn't change
    if (this.toKindString(this.specifier.kind) === this.selectedKind) {
      return;
    }
    
    const newKind = this.toSpecifierKind(this.selectedKind);
    this.specifier.kind = newKind;
  }

  updateValue(event: Event, targetValue: string, applyOffset = true): void {
    console.log(event);
    const target = event.target as HTMLInputElement;
    if (!target.checkValidity()) {
      console.log('Not updateing invalid change');
      return;
    }
    const newValue = Number(target.value) - (applyOffset ? this.offset : 0);

    console.log(`Updateing ${targetValue} to ${newValue}`);
    if (Object.hasOwn(this.specifier.kind as object, targetValue)) {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (this.specifier.kind as any)[targetValue] = newValue;
    }
    if (this.selectedKind === 'EveryNth' && targetValue.startsWith('EveryNth')) {
      const index = Number(targetValue.split('.')[1]);
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (this.specifier.kind as any)['EveryNth'][index] = newValue;
    }
   
  }

  /**
   * Returns the proper string to an unkown `SpecifierKind`.
   * @param unknownKind 
   * @returns Returns teh corresponding element from `SPECIFIER_KINDS` or `"None"` if no matching specifier was found.
   */
  toKindString(unknownKind: SpecifierKind): string {
    if (typeof unknownKind === 'string') {
      return unknownKind;
    }
    for (const kind of this.specifierKinds) {
      if (Object.hasOwn(unknownKind, kind)) {
        return kind;
      }
    }

    return 'None';
  }

  /**
   * Converts an element from `SPECIFIER_KINDS` into thge corresponding `SpecifierKind` or `None` if no matching was found
   * @param kindString value out of `SPECIFIER_KINDS`
   */
  toSpecifierKind(kindString: string): SpecifierKind {
    switch (kindString) {
      case 'All':
      case 'First':
      case 'Last':
        return kindString;
      case 'Nth':
        return { Nth: 0 };
      case 'BackNth':
        return { BackNth: 0 };
      case 'ExplicitNths':
        return { ExplicitNths: [] };
      case 'EveryNth':
        return { EveryNth: [0, 0] };
      case 'ExplicitList':
        return { ExplicitList: [] };
      case 'None':
      default:
        return 'None';
    }
  }

  specifierRange(end: 'min' | 'max', applyOffset = true): number {
    const value = (end === 'min' ?
      this.specifier.min :
      this.specifier.max) as object;
    
    let ret: number | undefined;
    if (typeof value === 'number') {
      ret = value;
    }
    if (isDay(value)) {
      ret = value.day;
    }
    if (isMonth(value)) {
      ret = value.month;
    }

    if (ret === undefined) {
      return 0;
    }
    return ret + (applyOffset ? this.offset : 0);
  }

  specifierValue(targetValue: string, applyOffset = true): number {
    if (Object.hasOwn(this.specifier.kind as object, targetValue)) {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      return (this.specifier.kind as any)[targetValue] as number + (applyOffset ? this.offset : 0);
    }
    if (this.selectedKind === 'EveryNth' && targetValue.startsWith('EveryNth')) {
      const index = Number(targetValue.split('.')[1]);
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      return (this.specifier.kind as any)['EveryNth'][index] as number + (applyOffset ? this.offset : 0);
    }
    return 0;
  }

  specifierList(targetList: string): number[] {
    if (Object.hasOwn(this.specifier.kind as object, targetList)) {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      return (this.specifier.kind as any)[targetList] as number[];
    }
    return [];
  }
}
