import * as React from 'react';
import Button from '@material-ui/core/Button';

export enum ButtonEvent {
  Next,
  Previous,
  Pause,
  Play,
}

export enum PlayState {
  Stopped,
  Paused,
  Playing,
}

type TransportButtonType = {
  click: (ButtonEvent) => void;
};

type TransportButtonProps = {
  click?: (ButtonEvent) => void;
  title: string;
  event?: ButtonEvent;
};

export class TransportButton extends React.Component<
  TransportButtonProps,
  TransportButtonType
> {
  constructor(props) {
    super(props);

    // This binding is necessary to make `this` work in the callback
    this.click = this.click.bind(this);
  }
  click(): void {
    this.props.click(this.props.event);
  }
  render() {
    return (
      <Button variant="contained" color="primary" onClick={this.click}>
        {' '}
        {this.props.title}
      </Button>
    );
  }
}

type PlayButtonProps = {
  play_state: PlayState;
  click: (any) => void;
};

export function PlayButton(props: PlayButtonProps) {
  if (props.play_state === PlayState.Stopped) {
    return (
      <TransportButton
        title="Play"
        event={ButtonEvent.Play}
        click={props.click}
      ></TransportButton>
    );
  }
  if (props.play_state === PlayState.Paused) {
    return (
      <TransportButton
        title="Play"
        event={ButtonEvent.Play}
        click={props.click}
      ></TransportButton>
    );
  }
  if (props.play_state === PlayState.Playing) {
    return (
      <TransportButton
        title="Pause"
        event={ButtonEvent.Pause}
        click={props.click}
      ></TransportButton>
    );
  }
  return <TransportButton title="Unspecified"></TransportButton>;
}
