function S(){}function C(t,n){for(const e in n)t[e]=n[e];return t}function B(t){return t()}function Z(){return Object.create(null)}function D(t){t.forEach(B)}function $(t){return typeof t=="function"}function tt(t,n){return t!=t?n==n:t!==n||t&&typeof t=="object"||typeof t=="function"}function nt(t){return Object.keys(t).length===0}function O(t,...n){if(t==null){for(const i of n)i(void 0);return S}const e=t.subscribe(...n);return e.unsubscribe?()=>e.unsubscribe():e}function et(t,n,e){t.$$.on_destroy.push(O(n,e))}function it(t,n,e,i){if(t){const l=k(t,n,e,i);return t[0](l)}}function k(t,n,e,i){return t[1]&&i?C(e.ctx.slice(),t[1](i(n))):e.ctx}function lt(t,n,e,i){if(t[2]&&i){const l=t[2](i(e));if(n.dirty===void 0)return l;if(typeof l=="object"){const s=[],c=Math.max(n.dirty.length,l.length);for(let o=0;o<c;o+=1)s[o]=n.dirty[o]|l[o];return s}return n.dirty|l}return n.dirty}function ct(t,n,e,i,l,s){if(l){const c=k(n,e,i,s);t.p(c,l)}}function rt(t){if(t.ctx.length>32){const n=[],e=t.ctx.length/32;for(let i=0;i<e;i++)n[i]=-1;return n}return-1}function st(t){return t??""}let m=!1;function ot(){m=!0}function ut(){m=!1}function q(t,n,e,i){for(;t<n;){const l=t+(n-t>>1);e(l)<=i?t=l+1:n=l}return t}function G(t){if(t.hydrate_init)return;t.hydrate_init=!0;let n=t.childNodes;if(t.nodeName==="HEAD"){const r=[];for(let u=0;u<n.length;u++){const a=n[u];a.claim_order!==void 0&&r.push(a)}n=r}const e=new Int32Array(n.length+1),i=new Int32Array(n.length);e[0]=-1;let l=0;for(let r=0;r<n.length;r++){const u=n[r].claim_order,a=(l>0&&n[e[l]].claim_order<=u?l+1:q(1,l,P=>n[e[P]].claim_order,u))-1;i[r]=e[a]+1;const w=a+1;e[w]=r,l=Math.max(w,l)}const s=[],c=[];let o=n.length-1;for(let r=e[l]+1;r!=0;r=i[r-1]){for(s.push(n[r-1]);o>=r;o--)c.push(n[o]);o--}for(;o>=0;o--)c.push(n[o]);s.reverse(),c.sort((r,u)=>r.claim_order-u.claim_order);for(let r=0,u=0;r<c.length;r++){for(;u<s.length&&c[r].claim_order>=s[u].claim_order;)u++;const a=u<s.length?s[u]:null;t.insertBefore(c[r],a)}}function I(t,n){if(m){for(G(t),(t.actual_end_child===void 0||t.actual_end_child!==null&&t.actual_end_child.parentNode!==t)&&(t.actual_end_child=t.firstChild);t.actual_end_child!==null&&t.actual_end_child.claim_order===void 0;)t.actual_end_child=t.actual_end_child.nextSibling;n!==t.actual_end_child?(n.claim_order!==void 0||n.parentNode!==t)&&t.insertBefore(n,t.actual_end_child):t.actual_end_child=n.nextSibling}else(n.parentNode!==t||n.nextSibling!==null)&&t.appendChild(n)}function z(t,n,e){t.insertBefore(n,e||null)}function F(t,n,e){m&&!e?I(t,n):(n.parentNode!==t||n.nextSibling!=e)&&t.insertBefore(n,e||null)}function g(t){t.parentNode&&t.parentNode.removeChild(t)}function at(t,n){for(let e=0;e<t.length;e+=1)t[e]&&t[e].d(n)}function M(t){return document.createElement(t)}function R(t){return document.createElementNS("http://www.w3.org/2000/svg",t)}function v(t){return document.createTextNode(t)}function ft(){return v(" ")}function _t(){return v("")}function dt(t,n,e,i){return t.addEventListener(n,e,i),()=>t.removeEventListener(n,e,i)}function ht(t){return function(n){return n.preventDefault(),t.call(this,n)}}function mt(t,n,e){e==null?t.removeAttribute(n):t.getAttribute(n)!==e&&t.setAttribute(n,e)}function pt(t){return t.dataset.svelteH}function yt(t){return Array.from(t.childNodes)}function H(t){t.claim_info===void 0&&(t.claim_info={last_index:0,total_claimed:0})}function L(t,n,e,i,l=!1){H(t);const s=(()=>{for(let c=t.claim_info.last_index;c<t.length;c++){const o=t[c];if(n(o)){const r=e(o);return r===void 0?t.splice(c,1):t[c]=r,l||(t.claim_info.last_index=c),o}}for(let c=t.claim_info.last_index-1;c>=0;c--){const o=t[c];if(n(o)){const r=e(o);return r===void 0?t.splice(c,1):t[c]=r,l?r===void 0&&t.claim_info.last_index--:t.claim_info.last_index=c,o}}return i()})();return s.claim_order=t.claim_info.total_claimed,t.claim_info.total_claimed+=1,s}function U(t,n,e,i){return L(t,l=>l.nodeName===n,l=>{const s=[];for(let c=0;c<l.attributes.length;c++){const o=l.attributes[c];e[o.name]||s.push(o.name)}s.forEach(c=>l.removeAttribute(c))},()=>i(n))}function bt(t,n,e){return U(t,n,e,M)}function W(t,n){return L(t,e=>e.nodeType===3,e=>{const i=""+n;if(e.data.startsWith(i)){if(e.data.length!==i.length)return e.splitText(i.length)}else e.data=i},()=>v(n),!0)}function gt(t){return W(t," ")}function N(t,n,e){for(let i=e;i<t.length;i+=1){const l=t[i];if(l.nodeType===8&&l.textContent.trim()===n)return i}return-1}function xt(t,n){const e=N(t,"HTML_TAG_START",0),i=N(t,"HTML_TAG_END",e+1);if(e===-1||i===-1)return new p(n);H(t);const l=t.splice(e,i-e+1);g(l[0]),g(l[l.length-1]);const s=l.slice(1,l.length-1);if(s.length===0)return new p(n);for(const c of s)c.claim_order=t.claim_info.total_claimed,t.claim_info.total_claimed+=1;return new p(n,s)}function Et(t,n){n=""+n,t.data!==n&&(t.data=n)}function vt(t,n,e,i){e==null?t.style.removeProperty(n):t.style.setProperty(n,e,i?"important":"")}function J(t,n,{bubbles:e=!1,cancelable:i=!1}={}){return new CustomEvent(t,{detail:n,bubbles:e,cancelable:i})}class K{is_svg=!1;e=void 0;n=void 0;t=void 0;a=void 0;constructor(n=!1){this.is_svg=n,this.e=this.n=null}c(n){this.h(n)}m(n,e,i=null){this.e||(this.is_svg?this.e=R(e.nodeName):this.e=M(e.nodeType===11?"TEMPLATE":e.nodeName),this.t=e.tagName!=="TEMPLATE"?e:e.content,this.c(n)),this.i(i)}h(n){this.e.innerHTML=n,this.n=Array.from(this.e.nodeName==="TEMPLATE"?this.e.content.childNodes:this.e.childNodes)}i(n){for(let e=0;e<this.n.length;e+=1)z(this.t,this.n[e],n)}p(n){this.d(),this.h(n),this.i(this.a)}d(){this.n.forEach(g)}}class p extends K{l=void 0;constructor(n=!1,e){super(n),this.e=this.n=null,this.l=e}c(n){this.l?this.n=this.l:super.c(n)}i(n){for(let e=0;e<this.n.length;e+=1)F(this.t,this.n[e],n)}}function Tt(t,n){return new t(n)}let h;function y(t){h=t}function T(){if(!h)throw new Error("Function called outside component initialization");return h}function wt(t){T().$$.on_mount.push(t)}function Nt(t){T().$$.after_update.push(t)}function At(){const t=T();return(n,e,{cancelable:i=!1}={})=>{const l=t.$$.callbacks[n];if(l){const s=J(n,e,{cancelable:i});return l.slice().forEach(c=>{c.call(t,s)}),!s.defaultPrevented}return!0}}const d=[],A=[];let _=[];const x=[],j=Promise.resolve();let E=!1;function Q(){E||(E=!0,j.then(X))}function kt(){return Q(),j}function V(t){_.push(t)}function Mt(t){x.push(t)}const b=new Set;let f=0;function X(){if(f!==0)return;const t=h;do{try{for(;f<d.length;){const n=d[f];f++,y(n),Y(n.$$)}}catch(n){throw d.length=0,f=0,n}for(y(null),d.length=0,f=0;A.length;)A.pop()();for(let n=0;n<_.length;n+=1){const e=_[n];b.has(e)||(b.add(e),e())}_.length=0}while(d.length);for(;x.length;)x.pop()();E=!1,b.clear(),y(t)}function Y(t){if(t.fragment!==null){t.update(),D(t.before_update);const n=t.dirty;t.dirty=[-1],t.fragment&&t.fragment.p(t.ctx,n),t.after_update.forEach(V)}}function Ht(t){const n=[],e=[];_.forEach(i=>t.indexOf(i)===-1?n.push(i):e.push(i)),e.forEach(i=>i()),_=n}export{pt as A,At as B,xt as C,dt as D,ht as E,at as F,st as G,p as H,D as I,V as J,$ as K,Mt as L,Z as M,X as N,nt as O,Ht as P,h as Q,y as R,B as S,d as T,Q as U,ot as V,ut as W,ft as a,Nt as b,gt as c,g as d,_t as e,M as f,bt as g,yt as h,F as i,mt as j,vt as k,v as l,W as m,Et as n,wt as o,A as p,Tt as q,it as r,tt as s,kt as t,ct as u,rt as v,lt as w,I as x,S as y,et as z};