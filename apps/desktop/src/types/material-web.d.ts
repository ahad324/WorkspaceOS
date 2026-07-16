import * as React from 'react';

declare module 'react' {
  namespace JSX {
    interface IntrinsicElements {
      'md-filled-button': React.DetailedHTMLProps<
        React.HTMLAttributes<HTMLElement> & {
          disabled?: boolean;
          href?: string;
          target?: string;
        },
        HTMLElement
      >;
      'md-outlined-button': React.DetailedHTMLProps<
        React.HTMLAttributes<HTMLElement> & {
          disabled?: boolean;
          href?: string;
          target?: string;
        },
        HTMLElement
      >;
      'md-elevated-button': React.DetailedHTMLProps<
        React.HTMLAttributes<HTMLElement> & {
          disabled?: boolean;
        },
        HTMLElement
      >;
      'md-text-button': React.DetailedHTMLProps<
        React.HTMLAttributes<HTMLElement> & {
          disabled?: boolean;
        },
        HTMLElement
      >;
      'md-outlined-text-field': React.DetailedHTMLProps<
        React.HTMLAttributes<HTMLElement> & {
          label?: string;
          value?: string;
          type?: string;
          required?: boolean;
          disabled?: boolean;
          placeholder?: string;
          error?: boolean;
          errorText?: string;
        },
        HTMLElement
      >;
      'md-outlined-select': React.DetailedHTMLProps<
        React.HTMLAttributes<HTMLElement> & {
          label?: string;
          value?: string;
          disabled?: boolean;
        },
        HTMLElement
      >;
      'md-select-option': React.DetailedHTMLProps<
        React.HTMLAttributes<HTMLElement> & {
          value?: string;
          selected?: boolean;
          headline?: string;
        },
        HTMLElement
      >;
      'md-checkbox': React.DetailedHTMLProps<
        React.HTMLAttributes<HTMLElement> & {
          checked?: boolean;
          disabled?: boolean;
        },
        HTMLElement
      >;
      'md-circular-progress': React.DetailedHTMLProps<
        React.HTMLAttributes<HTMLElement> & {
          indeterminate?: boolean;
          value?: number;
        },
        HTMLElement
      >;
      'md-tabs': React.DetailedHTMLProps<
        React.HTMLAttributes<HTMLElement> & {
          activeTabIndex?: number;
        },
        HTMLElement
      >;
      'md-primary-tab': React.DetailedHTMLProps<
        React.HTMLAttributes<HTMLElement> & {
          active?: boolean;
        },
        HTMLElement
      >;
      'md-icon': React.DetailedHTMLProps<React.HTMLAttributes<HTMLElement>, HTMLElement>;
      'md-icon-button': React.DetailedHTMLProps<
        React.HTMLAttributes<HTMLElement> & {
          disabled?: boolean;
        },
        HTMLElement
      >;
    }
  }
}
