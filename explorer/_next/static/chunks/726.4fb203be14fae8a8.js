"use strict";(self.webpackChunk_N_E=self.webpackChunk_N_E||[]).push([[726],{1726:(e,i,a)=>{a.r(i),a.d(i,{default:()=>Z});var n=a(7437),s=a(2265),l=a(1444),r=a(3719),t=a(6712),c=a(8473),o=a(2808),d=a(6673),g=a(4539),h=a(954),u=a(7045),x=a(6201),j=a(1960);function v(e){let[i,a]=(0,s.useState)(""),[t,c]=(0,s.useState)(!1),d=(0,l.I0)();function g(){e.onAskClose(),c(!1),a("")}function v(e){e.preventDefault(),(0,j.checkUrl)(i)?(g(),d((0,h.VT)(i))):c(!0)}return(0,n.jsxs)(u.Z,{show:e.show,onHide:g,size:"lg","aria-labelledby":"add-aggregator-title",centered:!0,children:[(0,n.jsx)(u.Z.Header,{closeButton:!0,children:(0,n.jsx)(u.Z.Title,{id:"add-aggregator-title",children:"New aggregator source"})}),(0,n.jsx)(u.Z.Body,{children:(0,n.jsx)(r.Z,{onSubmit:v,children:(0,n.jsxs)(x.Z,{children:[(0,n.jsx)(r.Z.Label,{children:"URL"}),(0,n.jsx)(r.Z.Control,{type:"url",value:i,onChange:e=>a(e.target.value),isInvalid:t,autoFocus:!0}),(0,n.jsx)(r.Z.Control.Feedback,{type:"invalid",children:"Invalid URL"})]})})}),(0,n.jsxs)(u.Z.Footer,{children:[(0,n.jsx)(o.Z,{variant:"secondary",onClick:g,children:"Close"}),(0,n.jsx)(o.Z,{variant:"primary",onClick:v,children:"Save"})]})]})}function Z(e){let[i,a]=(0,s.useState)(!1),u=(0,l.v9)(e=>e.settings.selectedAggregator),x=(0,l.v9)(e=>e.settings.availableAggregators),j=(0,l.v9)(e=>e.settings.canRemoveSelected),Z=(0,l.I0)();return(0,n.jsxs)(n.Fragment,{children:[(0,n.jsx)(v,{show:i,onAskClose:()=>a(!1)}),(0,n.jsxs)(r.Z.Group,{as:t.Z,className:e.className,children:[(0,n.jsx)(r.Z.Label,{children:"Aggregator:"}),(0,n.jsxs)(c.Z,{children:[(0,n.jsx)(o.Z,{variant:"outline-success",onClick:()=>a(!0),children:(0,n.jsx)("i",{className:"bi bi-plus-circle"})}),j&&(0,n.jsxs)(n.Fragment,{children:[(0,n.jsx)(o.Z,{variant:"outline-danger",onClick:()=>Z((0,h.OR)()),children:(0,n.jsx)("i",{className:"bi bi-dash-circle"})}),(0,n.jsx)(d.Z,{overlay:(0,n.jsx)(g.Z,{children:"Unofficial Aggregator"}),children:(0,n.jsx)(o.Z,{variant:"outline-warning",children:(0,n.jsx)("i",{className:"bi bi-exclamation-triangle"})})})]}),(0,n.jsx)(r.Z.Select,{value:u,onChange:e=>Z((0,h.VT)(e.target.value)),children:x.map((e,i)=>(0,n.jsx)("option",{value:e,children:e},"agg-"+i))}),(0,n.jsx)(d.Z,{overlay:(0,n.jsx)(g.Z,{children:"Copy url"}),children:(0,n.jsx)(o.Z,{variant:"outline-secondary",onClick:function(){window.isSecureContext&&u&&navigator.clipboard.writeText(u).then(()=>{})},children:(0,n.jsx)("i",{className:"bi bi-clipboard"})})})]})]})]})}}}]);