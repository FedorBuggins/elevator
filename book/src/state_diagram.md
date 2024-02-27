# State Diagram

```mermaid
stateDiagram-v2


[*] --> Stopped
Stopped --> Moving
Moving --> Opened
Opened --> Stopped
Opened --> Moving

```
